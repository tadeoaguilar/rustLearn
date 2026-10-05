use crate::sut::*;
use cluster::{Cluster, FakeCluster};
use crd::{App, AppSpec, Database, DatabaseSpec, Engine, Phase};
use database::FakeDbProvider;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Secret, Service};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{CustomResourceExt, ResourceExt};
use plan::{Change, Observed};
use reconcile::{Context, run_until_converged};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

fn meta(name: &str) -> ObjectMeta {
    ObjectMeta {
        name: Some(name.into()),
        namespace: Some("shop".into()),
        uid: Some(format!("uid-{name}")),
        generation: Some(1),
        ..ObjectMeta::default()
    }
}

fn spec() -> AppSpec {
    AppSpec {
        image: "ghcr.io/acme/web:1.4.2".into(),
        replicas: 3,
        port: 8080,
        env: BTreeMap::from([("RUST_LOG".into(), "info".into())]),
        config: BTreeMap::from([("app.toml".into(), "greeting = 1\n".into())]),
    }
}

fn app() -> App {
    App {
        metadata: meta("web"),
        spec: spec(),
        status: None,
    }
}

fn db() -> Database {
    Database {
        metadata: meta("orders-db"),
        spec: DatabaseSpec {
            engine: Engine::Postgres,
            version: "17".into(),
            storage_gb: 20,
        },
        status: None,
    }
}

fn context() -> (Context<FakeCluster>, Arc<FakeDbProvider>) {
    let provider = Arc::new(FakeDbProvider::new());
    (Context::new(FakeCluster::new(), provider.clone()), provider)
}

/// Like `kubectl apply`: the stored object gets a server-assigned uid.
async fn create<K: cluster::Object>(c: &FakeCluster, mut obj: K) {
    obj.meta_mut().uid = None;
    obj.meta_mut().generation = None;
    c.apply(&obj).await.unwrap();
}

async fn stored_app(c: &FakeCluster) -> App {
    c.get("shop", "web").await.unwrap().expect("app exists")
}

// ---- Exercise 1: the CRDs ---------------------------------------------------------------------

#[test]
fn ex1_crds_have_names_status_and_columns() {
    let crd = App::crd();
    assert_eq!(crd.metadata.name.as_deref(), Some("apps.rustlearn.dev"));
    assert_eq!(crd.spec.scope, "Namespaced");
    assert_eq!(crd.spec.names.short_names, Some(vec!["rapp".to_string()]));
    let version = &crd.spec.versions[0];
    assert!(
        version.subresources.as_ref().unwrap().status.is_some(),
        "status must be a subresource"
    );
    let columns: Vec<String> = version
        .additional_printer_columns
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|c| c.name)
        .collect();
    assert_eq!(columns, ["Image", "Desired", "Ready", "Phase"]);
    let db_columns = Database::crd().spec.versions[0]
        .additional_printer_columns
        .clone()
        .unwrap_or_default();
    assert_eq!(db_columns.len(), 3);
}

#[test]
fn ex1_schema_validation_reaches_the_api_server() {
    let schema = serde_json::to_value(&App::crd().spec.versions[0].schema).unwrap();
    let spec = &schema["openAPIV3Schema"]["properties"]["spec"]["properties"];
    assert_eq!(spec["replicas"]["minimum"], 0.0);
    assert_eq!(spec["replicas"]["maximum"], 50.0);
    assert_eq!(spec["port"]["maximum"], 65535.0);
    assert_eq!(spec["image"]["minLength"], 1);
    let db = serde_json::to_value(&Database::crd().spec.versions[0].schema).unwrap();
    assert_eq!(
        db["openAPIV3Schema"]["properties"]["spec"]["properties"]["storageGb"]["maximum"],
        1000.0
    );
}

#[test]
fn ex1_generated_yaml_matches_deploy_crds() {
    let yaml = crd::crds_yaml();
    let docs: Vec<serde_json::Value> = yaml
        .split("---\n")
        .map(|d| serde_yaml_ng::from_str(d).unwrap())
        .collect();
    assert_eq!(docs.len(), 2);
    assert_eq!(docs[1]["metadata"]["name"], "databases.rustlearn.dev");
    // The committed file must be regenerated whenever the solution's types
    // change. (Checked against the solution only: your own CRDs may differ in
    // descriptions or column order.)
    #[cfg(not(feature = "mine"))]
    {
        let committed =
            std::fs::read_to_string(format!("{}/crds.yaml", crate::sut::DEPLOY_DIR)).unwrap();
        assert_eq!(
            committed, yaml,
            "deploy/crds.yaml is stale: cargo run -p m22-kubernetes-operators-solution -- crds > deploy/crds.yaml"
        );
    }
}

#[test]
fn ex1_validate_app() {
    assert_eq!(crd::validate_app(&spec()), Ok(()));
    assert_eq!(
        crd::validate_app(&AppSpec {
            image: "ghcr.io/acme/web@sha256:abc".into(),
            ..spec()
        }),
        Ok(())
    );
    let mut bad = spec();
    bad.image = "localhost:5000/web".into(); // the colon is a registry port, not a tag
    bad.env.insert("1BAD".into(), "x".into());
    bad.config.insert("../etc/passwd".into(), "x".into());
    assert_eq!(crd::validate_app(&bad).unwrap_err().len(), 3);
    assert!(
        crd::validate_app(&AppSpec {
            image: "web:latest".into(),
            ..spec()
        })
        .is_err()
    );
}

// ---- Exercise 2: desired state ----------------------------------------------------------------

#[test]
fn ex2_children_are_labelled_and_owned() {
    let d = resources::desired(&app());
    assert_eq!(d.configmap.metadata.name.as_deref(), Some("web-config"));
    for meta in [
        &d.configmap.metadata,
        &d.deployment.metadata,
        &d.service.metadata,
    ] {
        assert_eq!(meta.namespace.as_deref(), Some("shop"));
        assert_eq!(
            meta.labels.as_ref().unwrap()["app.kubernetes.io/managed-by"],
            resources::MANAGER
        );
        let owner = &meta.owner_references.as_ref().expect("owner reference")[0];
        assert_eq!(
            (
                owner.kind.as_str(),
                owner.name.as_str(),
                owner.uid.as_str(),
                owner.controller
            ),
            ("App", "web", "uid-web", Some(true))
        );
    }
    let spec = d.deployment.spec.as_ref().unwrap();
    assert_eq!(spec.replicas, Some(3));
    let selector = spec.selector.match_labels.as_ref().unwrap();
    let pod_labels = spec
        .template
        .metadata
        .as_ref()
        .unwrap()
        .labels
        .as_ref()
        .unwrap();
    assert!(
        selector.iter().all(|(k, v)| pod_labels.get(k) == Some(v)),
        "the selector must match the pods"
    );
    let svc = d.service.spec.as_ref().unwrap();
    assert_eq!(svc.selector.as_ref(), Some(selector));
}

#[test]
fn ex2_pod_template() {
    let d = resources::desired_deployment(&app());
    let pod = d.spec.unwrap().template.spec.unwrap();
    let c = &pod.containers[0];
    assert_eq!(c.image.as_deref(), Some("ghcr.io/acme/web:1.4.2"));
    assert_eq!(c.env.as_ref().unwrap()[0].name, "RUST_LOG");
    let ready = c
        .readiness_probe
        .as_ref()
        .unwrap()
        .http_get
        .as_ref()
        .unwrap();
    assert_eq!(ready.path.as_deref(), Some("/readyz"));
    assert_eq!(
        c.liveness_probe
            .as_ref()
            .unwrap()
            .http_get
            .as_ref()
            .unwrap()
            .path
            .as_deref(),
        Some("/healthz")
    );
    assert_eq!(
        c.volume_mounts.as_ref().unwrap()[0].mount_path,
        resources::CONFIG_MOUNT
    );
    assert_eq!(pod.security_context.unwrap().run_as_non_root, Some(true));
}

#[test]
fn ex2_config_hash_changes_with_config_only() {
    let base = resources::config_hash(&spec().config);
    assert_eq!(base.len(), 16);
    assert_eq!(
        base,
        resources::config_hash(&spec().config),
        "deterministic"
    );
    let mut changed = spec().config;
    changed.insert("app.toml".into(), "greeting = 2\n".into());
    assert_ne!(base, resources::config_hash(&changed));
    let a = BTreeMap::from([("a".to_string(), "bc".to_string())]);
    let b = BTreeMap::from([("ab".to_string(), "c".to_string())]);
    assert_ne!(
        resources::config_hash(&a),
        resources::config_hash(&b),
        "keys and values must be delimited"
    );
}

// ---- Exercise 3: the plan --------------------------------------------------------------------

#[tokio::test]
async fn ex3_plan_ignores_server_defaults() {
    let desired = resources::desired(&app());
    assert_eq!(
        plan::plan(&desired, &Observed::default()),
        plan::Plan {
            configmap: Some(Change::Create),
            deployment: Some(Change::Create),
            service: Some(Change::Create)
        }
    );
    // Round-trip through the (fake) API server, which adds defaults.
    let c = FakeCluster::new();
    let observed = Observed {
        configmap: Some(c.apply(&desired.configmap).await.unwrap()),
        deployment: Some(c.apply(&desired.deployment).await.unwrap()),
        service: Some(c.apply(&desired.service).await.unwrap()),
    };
    assert_ne!(
        observed.deployment.as_ref().unwrap(),
        &desired.deployment,
        "the server changed the object..."
    );
    assert!(
        plan::plan(&desired, &observed).is_empty(),
        "...but nothing the operator owns differs"
    );
}

#[tokio::test]
async fn ex3_plan_names_what_changed() {
    let c = FakeCluster::new();
    let before = resources::desired(&app());
    let observed = Observed {
        configmap: Some(c.apply(&before.configmap).await.unwrap()),
        deployment: Some(c.apply(&before.deployment).await.unwrap()),
        service: Some(c.apply(&before.service).await.unwrap()),
    };
    let mut changed = app();
    changed.spec.replicas = 5;
    changed.spec.image = "ghcr.io/acme/web:1.5.0".into();
    changed
        .spec
        .config
        .insert("extra.toml".into(), "x = 1".into());
    let p = plan::plan(&resources::desired(&changed), &observed);
    assert_eq!(p.configmap, Some(Change::Update(vec!["data"])));
    assert_eq!(
        p.deployment,
        Some(Change::Update(vec!["replicas", "image", "config"]))
    );
    assert_eq!(p.service, None);
}

// ---- Exercise 4: status -------------------------------------------------------------------------

fn deployment_with(ready: i32, updated: i32, observed_generation: i64, stuck: bool) -> Deployment {
    serde_json::from_value(serde_json::json!({
        "metadata": {"name": "web", "generation": 2},
        "status": {
            "readyReplicas": ready, "updatedReplicas": updated, "observedGeneration": observed_generation,
            "conditions": if stuck { serde_json::json!([{"type": "Progressing", "status": "False", "reason": "ProgressDeadlineExceeded"}]) } else { serde_json::json!([]) }
        }
    }))
    .unwrap()
}

#[test]
fn ex4_phases_and_conditions() {
    let a = App {
        metadata: ObjectMeta {
            generation: Some(7),
            ..meta("web")
        },
        ..app()
    };
    let pending = status::app_status(&a, None);
    assert_eq!(
        (pending.phase, pending.observed_generation),
        (Phase::Pending, Some(7))
    );

    let rolling = status::app_status(&a, Some(&deployment_with(1, 3, 2, false)));
    assert_eq!(rolling.phase, Phase::Progressing);
    assert_eq!(rolling.conditions[0].message, "1/3 replicas ready");
    assert_eq!(rolling.conditions[0].status, "False");

    let ready = status::app_status(&a, Some(&deployment_with(3, 3, 2, false)));
    assert_eq!((ready.phase, ready.ready_replicas), (Phase::Ready, 3));
    assert_eq!(
        (
            ready.conditions[0].type_.as_str(),
            ready.conditions[0].status.as_str()
        ),
        ("Ready", "True")
    );

    let stale = status::app_status(&a, Some(&deployment_with(3, 3, 1, false)));
    assert_eq!(
        stale.phase,
        Phase::Progressing,
        "the Deployment controller hasn't seen the latest spec yet"
    );

    assert_eq!(
        status::app_status(&a, Some(&deployment_with(0, 0, 2, true))).phase,
        Phase::Degraded
    );
}

// ---- Exercise 5: finalizers and credentials ----------------------------------------------------

#[test]
fn ex5_finalizer_helpers_and_secret() {
    let others = vec!["other.io/keep".to_string()];
    let added = database::with_finalizer(&others);
    assert_eq!(added, ["other.io/keep", database::FINALIZER]);
    assert_eq!(database::with_finalizer(&added), added, "no duplicates");
    assert_eq!(
        database::without_finalizer(&added),
        others,
        "other controllers' finalizers stay"
    );

    let endpoint = database::Endpoint {
        host: "db.internal".into(),
        port: 5432,
    };
    let secret = database::credentials_secret(&db(), &endpoint, "pw123");
    assert_eq!(
        secret.metadata.name.as_deref(),
        Some("orders-db-credentials")
    );
    assert_eq!(
        secret.metadata.owner_references.as_ref().unwrap()[0].kind,
        "Database"
    );
    assert_eq!(database::password_from(&secret).as_deref(), Some("pw123"));
    let url = String::from_utf8(secret.data.as_ref().unwrap()["url"].0.clone()).unwrap();
    assert_eq!(url, "postgres://orders_db:pw123@db.internal:5432/orders_db");
    let p = database::generate_password(24);
    assert!(p.len() == 24 && p.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_ne!(p, database::generate_password(24));
}

#[tokio::test]
async fn ex5_database_lifecycle_with_finalizer() {
    let (ctx, provider) = context();
    create(&ctx.cluster, db()).await;
    run_until_converged(&ctx, 10).await.expect("converges");

    let stored: Database = ctx.cluster.get("shop", "orders-db").await.unwrap().unwrap();
    assert!(database::has_finalizer(&stored));
    let status = stored.status.unwrap();
    assert_eq!(status.phase, Phase::Ready);
    assert_eq!(
        status.endpoint.as_deref(),
        Some("shop-orders-db.db.internal:5432")
    );
    assert_eq!(provider.databases(), ["shop/orders-db"]);
    let secret: Secret = ctx
        .cluster
        .get("shop", "orders-db-credentials")
        .await
        .unwrap()
        .unwrap();
    let password = database::password_from(&secret).unwrap();

    // Reconciling again keeps the password.
    ctx.cluster
        .edit::<Database>("shop", "orders-db", |d| d.spec.storage_gb = 40)
        .unwrap();
    run_until_converged(&ctx, 10).await.unwrap();
    let secret: Secret = ctx
        .cluster
        .get("shop", "orders-db-credentials")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        database::password_from(&secret),
        Some(password),
        "never rotate on every reconcile"
    );

    // Delete: blocked by the finalizer until the operator has cleaned up.
    ctx.cluster
        .delete::<Database>("shop", "orders-db")
        .await
        .unwrap();
    let pending: Database = ctx
        .cluster
        .get("shop", "orders-db")
        .await
        .unwrap()
        .expect("still there");
    assert!(pending.metadata.deletion_timestamp.is_some());
    assert_eq!(provider.databases().len(), 1);
    run_until_converged(&ctx, 10).await.unwrap();
    assert!(
        ctx.cluster
            .get::<Database>("shop", "orders-db")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        provider.databases().is_empty(),
        "the external database was deleted"
    );
    assert!(
        ctx.cluster
            .get::<Secret>("shop", "orders-db-credentials")
            .await
            .unwrap()
            .is_none(),
        "garbage-collected"
    );
}

#[tokio::test]
async fn ex5_permanent_errors_go_to_status_not_retries() {
    let (ctx, _) = context();
    create(&ctx.cluster, db()).await;
    run_until_converged(&ctx, 10).await.unwrap();
    ctx.cluster
        .edit::<Database>("shop", "orders-db", |d| d.spec.storage_gb = 5)
        .unwrap();
    run_until_converged(&ctx, 10)
        .await
        .expect("a permanent error still converges");
    let status = ctx
        .cluster
        .get::<Database>("shop", "orders-db")
        .await
        .unwrap()
        .unwrap()
        .status
        .unwrap();
    assert_eq!(status.phase, Phase::Degraded);
    assert!(status.message.unwrap().contains("shrink"));
}

// ---- Exercise 6: reconciling Apps -----------------------------------------------------------------

#[tokio::test]
async fn ex6_app_converges_and_then_stays_quiet() {
    let (ctx, _) = context();
    create(&ctx.cluster, app()).await;
    let rounds = run_until_converged(&ctx, 20).await.expect("converges");
    assert!(rounds >= 2, "pods become ready over several rounds");
    assert_eq!(ctx.cluster.count::<Deployment>(), 1);
    assert_eq!(ctx.cluster.count::<Service>(), 1);
    assert_eq!(ctx.cluster.count::<ConfigMap>(), 1);
    let status = stored_app(&ctx.cluster).await.status.unwrap();
    assert_eq!(
        (
            status.phase,
            status.ready_replicas,
            status.observed_generation
        ),
        (Phase::Ready, 3, Some(1))
    );

    let writes = ctx.cluster.writes();
    assert_eq!(run_until_converged(&ctx, 5).await, Ok(1));
    assert_eq!(
        ctx.cluster.writes(),
        writes,
        "a converged reconcile writes nothing"
    );
}

#[tokio::test]
async fn ex6_config_change_rolls_pods_and_drift_is_repaired() {
    let (ctx, _) = context();
    create(&ctx.cluster, app()).await;
    run_until_converged(&ctx, 20).await.unwrap();
    let hash = |d: &Deployment| {
        d.spec
            .as_ref()
            .unwrap()
            .template
            .metadata
            .as_ref()
            .unwrap()
            .annotations
            .as_ref()
            .unwrap()[resources::CONFIG_HASH]
            .clone()
    };
    let before: Deployment = ctx.cluster.get("shop", "web").await.unwrap().unwrap();

    ctx.cluster
        .edit::<App>("shop", "web", |a| {
            a.spec
                .config
                .insert("app.toml".into(), "greeting = 2\n".into());
        })
        .unwrap();
    run_until_converged(&ctx, 20).await.unwrap();
    let after: Deployment = ctx.cluster.get("shop", "web").await.unwrap().unwrap();
    assert_ne!(
        hash(&before),
        hash(&after),
        "pod template changed -> rolling update"
    );
    let cm: ConfigMap = ctx
        .cluster
        .get("shop", "web-config")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cm.data.unwrap()["app.toml"], "greeting = 2\n");
    assert_eq!(
        stored_app(&ctx.cluster)
            .await
            .status
            .unwrap()
            .observed_generation,
        Some(2)
    );

    // Someone runs `kubectl scale deployment web --replicas=10`.
    ctx.cluster
        .edit::<Deployment>("shop", "web", |d| {
            d.spec.as_mut().unwrap().replicas = Some(10)
        })
        .unwrap();
    run_until_converged(&ctx, 20).await.unwrap();
    let d: Deployment = ctx.cluster.get("shop", "web").await.unwrap().unwrap();
    assert_eq!(
        d.spec.unwrap().replicas,
        Some(3),
        "the operator owns replicas"
    );
}

#[tokio::test]
async fn ex6_bad_images_invalid_specs_and_deletion() {
    let (ctx, _) = context();
    create(
        &ctx.cluster,
        App {
            spec: AppSpec {
                image: "ghcr.io/acme/web:broken".into(),
                ..spec()
            },
            ..app()
        },
    )
    .await;
    run_until_converged(&ctx, 20).await.unwrap();
    assert_eq!(
        stored_app(&ctx.cluster).await.status.unwrap().phase,
        Phase::Degraded
    );

    let (ctx2, _) = context();
    create(
        &ctx2.cluster,
        App {
            spec: AppSpec {
                image: "web".into(),
                ..spec()
            },
            ..app()
        },
    )
    .await;
    run_until_converged(&ctx2, 20).await.unwrap();
    let status = stored_app(&ctx2.cluster).await.status.unwrap();
    assert_eq!(status.phase, Phase::Degraded);
    assert_eq!(status.conditions[0].reason, "InvalidSpec");
    assert_eq!(
        ctx2.cluster.count::<Deployment>(),
        0,
        "nothing is created for an invalid spec"
    );

    ctx.cluster.delete::<App>("shop", "web").await.unwrap();
    assert_eq!(
        ctx.cluster.count::<Deployment>()
            + ctx.cluster.count::<Service>()
            + ctx.cluster.count::<ConfigMap>(),
        0,
        "owner references: children are garbage-collected"
    );
}

// ---- Exercise 7: retries and metrics --------------------------------------------------------------

#[test]
fn ex7_backoff_per_object() {
    let mut b = metrics::Backoff::new(Duration::from_secs(1), Duration::from_secs(10));
    let delays: Vec<u64> = (0..5).map(|_| b.failure("shop/web").as_secs()).collect();
    assert_eq!(delays, [1, 2, 4, 8, 10]);
    assert_eq!(
        b.failure("shop/other").as_secs(),
        1,
        "each object has its own backoff"
    );
    b.success("shop/web");
    assert_eq!(b.failures("shop/web"), 0);
    assert_eq!(b.failure("shop/web").as_secs(), 1);
    for _ in 0..100 {
        b.failure("shop/web"); // no overflow
    }
}

#[tokio::test]
async fn ex7_transient_failures_are_retried_and_counted() {
    let (ctx, provider) = context();
    create(&ctx.cluster, app()).await;
    create(&ctx.cluster, db()).await;
    ctx.cluster.fail_next(3);
    provider.fail_next(2);
    run_until_converged(&ctx, 30)
        .await
        .expect("converges despite failures");
    assert_eq!(
        stored_app(&ctx.cluster).await.status.unwrap().phase,
        Phase::Ready
    );
    assert!(ctx.metrics.reconciles("App", false) + ctx.metrics.reconciles("Database", false) >= 2);
    assert!(ctx.metrics.reconciles("App", true) >= 1);

    let text = ctx.metrics.render();
    assert!(text.contains("# TYPE rustlearn_operator_reconciles_total counter"));
    assert!(
        text.contains("rustlearn_operator_reconciles_total{kind=\"App\",result=\"ok\"}"),
        "{text}"
    );
    assert!(
        text.contains("rustlearn_operator_reconcile_duration_seconds_count{kind=\"Database\"}"),
        "{text}"
    );
}

// ---- Bonus: the deploy/ examples are valid resources ------------------------------------------

#[test]
fn bonus_example_manifests_parse_and_validate() {
    let dir = format!("{}/examples", crate::sut::DEPLOY_DIR);
    let app: App =
        serde_yaml_ng::from_str(&std::fs::read_to_string(format!("{dir}/app.yaml")).unwrap())
            .unwrap();
    assert_eq!(crd::validate_app(&app.spec), Ok(()));
    assert_eq!(app.name_any(), "web");
    let db: Database =
        serde_yaml_ng::from_str(&std::fs::read_to_string(format!("{dir}/database.yaml")).unwrap())
            .unwrap();
    assert_eq!(db.spec.engine, Engine::Postgres);
}
