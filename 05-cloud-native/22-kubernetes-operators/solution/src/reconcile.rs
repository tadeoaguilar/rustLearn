//! Exercise 6: the reconcilers.
//!
//! A reconcile looks at *one* object, compares the world with what it
//! should be, and makes one step towards it. It's called again whenever the
//! object or anything it owns changes, and periodically anyway. So it must
//! be **idempotent** (running it twice is the same as once) and **level-
//! based** (it reads the current state; it never asks "what event was this?").

use crate::cluster::{Cluster, ClusterError, FakeCluster};
use crate::crd::{App, Database, DatabaseStatus, Phase};
use crate::database::{self, DbProvider, ProviderError};
use crate::metrics::{Backoff, Metrics};
use crate::plan::{Observed, plan};
use crate::resources::desired;
use crate::status::{app_status, invalid_status};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use kube::ResourceExt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct Context<C> {
    pub cluster: C,
    pub provider: Arc<dyn DbProvider>,
    pub metrics: Metrics,
    pub backoff: Mutex<Backoff>,
}

impl<C: Cluster> Context<C> {
    pub fn new(cluster: C, provider: Arc<dyn DbProvider>) -> Context<C> {
        Context {
            cluster,
            provider,
            metrics: Metrics::new(),
            backoff: Mutex::new(Backoff::new(
                Duration::from_secs(1),
                Duration::from_secs(300),
            )),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Cluster(#[from] ClusterError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
}

/// What to do after a successful reconcile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Check again after this long, even if nothing changes.
    Requeue(Duration),
    /// Nothing to do until the object changes.
    AwaitChange,
}

/// "namespace/name": the key for backoff and logs.
pub fn key(obj: &impl ResourceExt) -> String {
    format!("{}/{}", obj.namespace().unwrap_or_default(), obj.name_any())
}

pub async fn reconcile_app<C: Cluster>(app: &App, ctx: &Context<C>) -> Result<Outcome, Error> {
    // Children have owner references: the garbage collector deletes them.
    if app.metadata.deletion_timestamp.is_some() {
        return Ok(Outcome::AwaitChange);
    }
    let ns = app.namespace().unwrap_or_else(|| "default".into());
    let name = app.name_any();
    let cluster = &ctx.cluster;

    if let Err(problems) = crate::crd::validate_app(&app.spec) {
        let status = invalid_status(app, &problems);
        if app.status.as_ref() != Some(&status) {
            cluster
                .apply_status(&App {
                    status: Some(status),
                    ..app.clone()
                })
                .await?;
        }
        return Ok(Outcome::AwaitChange); // retrying won't fix a bad spec
    }

    let want = desired(app);
    let observed = Observed {
        configmap: cluster
            .get::<ConfigMap>(&ns, &format!("{name}-config"))
            .await?,
        deployment: cluster.get::<Deployment>(&ns, &name).await?,
        service: cluster.get::<Service>(&ns, &name).await?,
    };
    let steps = plan(&want, &observed);
    // The ConfigMap first: pods of the new Deployment mount it.
    if steps.configmap.is_some() {
        cluster.apply(&want.configmap).await?;
    }
    let deployment = match steps.deployment {
        Some(_) => Some(cluster.apply(&want.deployment).await?),
        None => observed.deployment,
    };
    if steps.service.is_some() {
        cluster.apply(&want.service).await?;
    }

    let status = app_status(app, deployment.as_ref());
    let ready = status.phase == Phase::Ready;
    if app.status.as_ref() != Some(&status) {
        cluster
            .apply_status(&App {
                status: Some(status),
                ..app.clone()
            })
            .await?;
    }
    // While rolling out, look again soon; once ready, a periodic resync.
    Ok(Outcome::Requeue(if ready {
        Duration::from_secs(300)
    } else {
        Duration::from_secs(5)
    }))
}

async fn set_database_status<C: Cluster>(
    db: &Database,
    ctx: &Context<C>,
    status: DatabaseStatus,
) -> Result<(), Error> {
    if db.status.as_ref() != Some(&status) {
        ctx.cluster
            .apply_status(&Database {
                status: Some(status),
                ..db.clone()
            })
            .await?;
    }
    Ok(())
}

pub async fn reconcile_database<C: Cluster>(
    db: &Database,
    ctx: &Context<C>,
) -> Result<Outcome, Error> {
    let ns = db.namespace().unwrap_or_else(|| "default".into());
    let name = db.name_any();
    let id = key(db);

    if db.metadata.deletion_timestamp.is_some() {
        if database::has_finalizer(db) {
            ctx.provider.delete(&id)?; // fails -> retried; the finalizer stays until it works
            ctx.cluster
                .set_finalizers::<Database>(
                    &ns,
                    &name,
                    database::without_finalizer(db.finalizers()),
                )
                .await?;
        }
        return Ok(Outcome::AwaitChange);
    }

    // Finalizer before anything external exists.
    if !database::has_finalizer(db) {
        ctx.cluster
            .set_finalizers::<Database>(&ns, &name, database::with_finalizer(db.finalizers()))
            .await?;
    }

    let endpoint = match ctx.provider.ensure(&id, &db.spec) {
        Ok(e) => e,
        Err(ProviderError::Invalid(message)) => {
            let status = DatabaseStatus {
                phase: Phase::Degraded,
                message: Some(message),
                ..db.status.clone().unwrap_or_default()
            };
            set_database_status(db, ctx, status).await?;
            return Ok(Outcome::AwaitChange);
        }
        Err(e) => return Err(e.into()),
    };

    let existing = ctx
        .cluster
        .get::<Secret>(&ns, &database::secret_name(db))
        .await?;
    let password = existing
        .as_ref()
        .and_then(database::password_from)
        .unwrap_or_else(|| database::generate_password(24));
    ctx.cluster
        .apply(&database::credentials_secret(db, &endpoint, &password))
        .await?;

    let status = DatabaseStatus {
        phase: Phase::Ready,
        endpoint: Some(format!("{}:{}", endpoint.host, endpoint.port)),
        secret_name: Some(database::secret_name(db)),
        message: None,
    };
    set_database_status(db, ctx, status).await?;
    Ok(Outcome::Requeue(Duration::from_secs(600)))
}

/// Records metrics and backoff around one reconcile.
async fn observe<F: Future<Output = Result<Outcome, Error>>>(
    metrics: &Metrics,
    backoff: &Mutex<Backoff>,
    kind: &str,
    key: &str,
    run: F,
) -> Result<Outcome, Duration> {
    let started = Instant::now();
    let result = run.await;
    metrics.record(kind, result.is_ok(), started.elapsed());
    match result {
        Ok(outcome) => {
            backoff.lock().unwrap().success(key);
            Ok(outcome)
        }
        Err(_) => Err(backoff.lock().unwrap().failure(key)),
    }
}

/// Test harness: reconcile every App and Database, then let the built-in
/// controllers react, until a whole round changes nothing. Returns the
/// number of rounds, or `Err(max_rounds)` if it never settles. Failed
/// reconciles are retried in the next round (the harness doesn't sleep out
/// the backoff).
pub async fn run_until_converged(
    ctx: &Context<FakeCluster>,
    max_rounds: usize,
) -> Result<usize, usize> {
    for round in 1..=max_rounds {
        let before = ctx.cluster.writes();
        let mut failed = false;
        match ctx.cluster.list::<App>().await {
            Ok(apps) => {
                for app in apps {
                    failed |= observe(
                        &ctx.metrics,
                        &ctx.backoff,
                        "App",
                        &key(&app),
                        reconcile_app(&app, ctx),
                    )
                    .await
                    .is_err();
                }
            }
            Err(_) => failed = true,
        }
        match ctx.cluster.list::<Database>().await {
            Ok(dbs) => {
                for db in dbs {
                    failed |= observe(
                        &ctx.metrics,
                        &ctx.backoff,
                        "Database",
                        &key(&db),
                        reconcile_database(&db, ctx),
                    )
                    .await
                    .is_err();
                }
            }
            Err(_) => failed = true,
        }
        ctx.cluster.run_builtin_controllers();
        if !failed && ctx.cluster.writes() == before {
            return Ok(round);
        }
    }
    Err(max_rounds)
}
