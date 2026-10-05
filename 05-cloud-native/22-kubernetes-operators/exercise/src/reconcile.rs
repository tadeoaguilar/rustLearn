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
    todo!("Exercise 6")
}

async fn set_database_status<C: Cluster>(
    db: &Database,
    ctx: &Context<C>,
    status: DatabaseStatus,
) -> Result<(), Error> {
    todo!("Exercise 5")
}

pub async fn reconcile_database<C: Cluster>(
    db: &Database,
    ctx: &Context<C>,
) -> Result<Outcome, Error> {
    todo!("Exercise 5")
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

/// PROVIDED test harness: reconcile every App and Database, then let the built-in
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
