//! Running the operator against a real cluster.
//!
//! PROVIDED in the exercise crate. `KubeCluster` implements the same
//! `Cluster` trait as `FakeCluster`, with kube-rs API calls; `run` wires the
//! reconcilers into kube-rs `Controller`s, which watch the API server and
//! call them whenever an App, a Database, or something they own changes.
//!
//! ```text
//! kind create cluster        # or Docker Desktop's Kubernetes, minikube, k3d...
//! cargo run -p m22-kubernetes-operators-solution -- crds | kubectl apply -f -
//! cargo run -p m22-kubernetes-operators-solution -- run
//! kubectl apply -f 05-cloud-native/22-kubernetes-operators/deploy/examples/
//! ```

use crate::cluster::{Cluster, ClusterError, Object};
use crate::crd::{App, Database};
use crate::database::FakeDbProvider;
use crate::reconcile::{Context, Error, Outcome, key, reconcile_app, reconcile_database};
use futures::StreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use kube::api::{Api, DeleteParams, Patch, PatchParams};
use kube::runtime::controller::{Action, Controller};
use kube::runtime::watcher;
use kube::{Client, ResourceExt};
use serde_json::json;
use std::sync::Arc;

pub const FIELD_MANAGER: &str = "rustlearn-operator";

pub struct KubeCluster {
    pub client: Client,
}

fn convert(e: kube::Error) -> ClusterError {
    match e {
        kube::Error::Api(status) if status.code == 404 => {
            ClusterError::NotFound(status.message.clone())
        }
        other => ClusterError::Api(other.to_string()),
    }
}

impl KubeCluster {
    fn api<K: Object>(&self, namespace: &str) -> Api<K> {
        Api::namespaced(self.client.clone(), namespace)
    }
}

impl Cluster for KubeCluster {
    async fn get<K: Object>(&self, namespace: &str, name: &str) -> Result<Option<K>, ClusterError> {
        self.api::<K>(namespace)
            .get_opt(name)
            .await
            .map_err(convert)
    }

    async fn list<K: Object>(&self) -> Result<Vec<K>, ClusterError> {
        let api: Api<K> = Api::all(self.client.clone());
        Ok(api.list(&Default::default()).await.map_err(convert)?.items)
    }

    async fn apply<K: Object>(&self, obj: &K) -> Result<K, ClusterError> {
        let ns = obj.namespace().unwrap_or_else(|| "default".into());
        // `force`: this operator owns these fields, even if someone else last wrote them.
        let params = PatchParams::apply(FIELD_MANAGER).force();
        self.api::<K>(&ns)
            .patch(&obj.name_any(), &params, &Patch::Apply(obj))
            .await
            .map_err(convert)
    }

    async fn apply_status<K: Object>(&self, obj: &K) -> Result<(), ClusterError> {
        let ns = obj.namespace().unwrap_or_else(|| "default".into());
        let value = serde_json::to_value(obj).map_err(|e| ClusterError::Api(e.to_string()))?;
        let patch = json!({ "apiVersion": K::api_version(&()), "kind": K::kind(&()), "status": value["status"] });
        let params = PatchParams::apply(FIELD_MANAGER).force();
        self.api::<K>(&ns)
            .patch_status(&obj.name_any(), &params, &Patch::Apply(&patch))
            .await
            .map_err(convert)?;
        Ok(())
    }

    async fn set_finalizers<K: Object>(
        &self,
        namespace: &str,
        name: &str,
        finalizers: Vec<String>,
    ) -> Result<(), ClusterError> {
        let patch = json!({ "metadata": { "finalizers": finalizers } });
        self.api::<K>(namespace)
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(convert)?;
        Ok(())
    }

    async fn delete<K: Object>(&self, namespace: &str, name: &str) -> Result<(), ClusterError> {
        match self
            .api::<K>(namespace)
            .delete(name, &DeleteParams::default())
            .await
        {
            Ok(_) => Ok(()),
            Err(kube::Error::Api(s)) if s.code == 404 => Ok(()),
            Err(e) => Err(convert(e)),
        }
    }
}

fn to_action(outcome: Outcome) -> Action {
    match outcome {
        Outcome::Requeue(d) => Action::requeue(d),
        Outcome::AwaitChange => Action::await_change(),
    }
}

/// Watches Apps and Databases until Ctrl-C / SIGTERM.
pub async fn run(client: Client) {
    let ctx = Arc::new(Context::new(
        KubeCluster {
            client: client.clone(),
        },
        Arc::new(FakeDbProvider::new()),
    ));
    let wc = watcher::Config::default();

    let apps = Controller::new(Api::<App>::all(client.clone()), wc.clone())
        .owns(Api::<Deployment>::all(client.clone()), wc.clone())
        .owns(Api::<Service>::all(client.clone()), wc.clone())
        .owns(Api::<ConfigMap>::all(client.clone()), wc.clone())
        .shutdown_on_signal()
        .run(
            |app: Arc<App>, ctx: Arc<Context<KubeCluster>>| async move {
                let started = std::time::Instant::now();
                let result = reconcile_app(&app, &ctx).await;
                ctx.metrics.record("App", result.is_ok(), started.elapsed());
                if result.is_ok() {
                    ctx.backoff.lock().unwrap().success(&key(app.as_ref()));
                }
                result.map(to_action)
            },
            |app: Arc<App>, err: &Error, ctx: Arc<Context<KubeCluster>>| {
                let delay = ctx.backoff.lock().unwrap().failure(&key(app.as_ref()));
                eprintln!("App {}: {err}; retrying in {delay:?}", key(app.as_ref()));
                Action::requeue(delay)
            },
            ctx.clone(),
        )
        .for_each(|result| async move {
            match result {
                Ok((obj, action)) => eprintln!("reconciled App {} -> {action:?}", obj.name),
                Err(e) => eprintln!("controller error: {e}"),
            }
        });

    let databases = Controller::new(Api::<Database>::all(client.clone()), wc.clone())
        .owns(Api::<Secret>::all(client), wc)
        .shutdown_on_signal()
        .run(
            |db: Arc<Database>, ctx: Arc<Context<KubeCluster>>| async move {
                let started = std::time::Instant::now();
                let result = reconcile_database(&db, &ctx).await;
                ctx.metrics
                    .record("Database", result.is_ok(), started.elapsed());
                if result.is_ok() {
                    ctx.backoff.lock().unwrap().success(&key(db.as_ref()));
                }
                result.map(to_action)
            },
            |db: Arc<Database>, err: &Error, ctx: Arc<Context<KubeCluster>>| {
                let delay = ctx.backoff.lock().unwrap().failure(&key(db.as_ref()));
                eprintln!(
                    "Database {}: {err}; retrying in {delay:?}",
                    key(db.as_ref())
                );
                Action::requeue(delay)
            },
            ctx.clone(),
        )
        .for_each(|result| async move {
            match result {
                Ok((obj, action)) => eprintln!("reconciled Database {} -> {action:?}", obj.name),
                Err(e) => eprintln!("controller error: {e}"),
            }
        });

    tokio::join!(apps, databases);
    eprintln!("{}", ctx.metrics.render());
}
