// Reference solution for 22-kubernetes-operators.
//
//     cargo run -p m22-kubernetes-operators-solution -- demo   # the whole lifecycle, on FakeCluster
//     cargo run -p m22-kubernetes-operators-solution -- crds   # print the CRDs (pipe into kubectl apply -f -)
//     cargo run -p m22-kubernetes-operators-solution -- run    # against the cluster in your kubeconfig

use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use m22_kubernetes_operators_solution::cluster::{Cluster, FakeCluster};
use m22_kubernetes_operators_solution::crd::{
    App, AppSpec, Database, DatabaseSpec, Engine, crds_yaml,
};
use m22_kubernetes_operators_solution::database::FakeDbProvider;
use m22_kubernetes_operators_solution::reconcile::{Context, run_until_converged};
use std::collections::BTreeMap;
use std::sync::Arc;

fn meta(name: &str) -> ObjectMeta {
    ObjectMeta {
        name: Some(name.into()),
        namespace: Some("shop".into()),
        ..ObjectMeta::default()
    }
}

async fn show(ctx: &Context<FakeCluster>, label: &str) {
    let rounds = run_until_converged(ctx, 20).await;
    let app: App = ctx
        .cluster
        .get("shop", "web")
        .await
        .unwrap()
        .unwrap_or_else(|| {
            App::new(
                "gone",
                AppSpec {
                    image: String::new(),
                    replicas: 0,
                    port: 0,
                    env: BTreeMap::new(),
                    config: BTreeMap::new(),
                },
            )
        });
    let status = app.status.unwrap_or_default();
    println!(
        "{label:<38} converged in {rounds:?} rounds | App phase {:?}, {} ready | {} writes so far",
        status.phase,
        status.ready_replicas,
        ctx.cluster.writes()
    );
}

async fn demo() {
    let provider = Arc::new(FakeDbProvider::new());
    let ctx = Context::new(FakeCluster::new(), provider.clone());
    let c = &ctx.cluster;

    let spec = AppSpec {
        image: "ghcr.io/acme/web:1.4.2".into(),
        replicas: 3,
        port: 8080,
        env: BTreeMap::from([("RUST_LOG".into(), "info".into())]),
        config: BTreeMap::from([("app.toml".into(), "greeting = \"hello\"\n".into())]),
    };
    c.apply(&App {
        metadata: meta("web"),
        spec,
        status: None,
    })
    .await
    .unwrap();
    show(&ctx, "kubectl apply app/web (3 replicas)").await;
    show(&ctx, "nothing changed").await;

    c.edit::<App>("shop", "web", |a| {
        a.spec
            .config
            .insert("app.toml".into(), "greeting = \"hi\"\n".into())
            .map(drop)
            .unwrap_or(())
    })
    .unwrap();
    show(&ctx, "config edited (pods roll)").await;

    c.edit::<Deployment>("shop", "web", |d| {
        d.spec.as_mut().unwrap().replicas = Some(10)
    })
    .unwrap();
    show(&ctx, "someone scaled the Deployment to 10").await;
    let d: Deployment = c.get("shop", "web").await.unwrap().unwrap();
    println!(
        "{:<38} Deployment replicas back to {:?}",
        "",
        d.spec.unwrap().replicas
    );

    c.edit::<App>("shop", "web", |a| {
        a.spec.image = "ghcr.io/acme/web:broken".into()
    })
    .unwrap();
    show(&ctx, "bad image").await;
    c.edit::<App>("shop", "web", |a| a.spec.image = "ghcr.io/acme/web".into())
        .unwrap();
    show(&ctx, "image without a tag (invalid spec)").await;
    let app: App = c.get("shop", "web").await.unwrap().unwrap();
    println!("{:<38} {}", "", app.status.unwrap().conditions[0].message);

    let db = Database {
        metadata: meta("orders-db"),
        spec: DatabaseSpec {
            engine: Engine::Postgres,
            version: "17".into(),
            storage_gb: 20,
        },
        status: None,
    };
    c.apply(&db).await.unwrap();
    println!(
        "\nDatabase converged in {:?} rounds",
        run_until_converged(&ctx, 20).await
    );
    let db: Database = c.get("shop", "orders-db").await.unwrap().unwrap();
    println!(
        "  finalizers {:?}, status {:?}",
        db.metadata.finalizers, db.status
    );
    println!(
        "  provider has {:?}; Secret exists: {}",
        provider.databases(),
        c.get::<Secret>("shop", "orders-db-credentials")
            .await
            .unwrap()
            .is_some()
    );

    c.delete::<Database>("shop", "orders-db").await.unwrap();
    let pending: Option<Database> = c.get("shop", "orders-db").await.unwrap();
    println!(
        "kubectl delete database: still there with deletionTimestamp = {}",
        pending.is_some()
    );
    run_until_converged(&ctx, 20).await.unwrap();
    println!(
        "  after the operator ran: Database exists {}, Secret exists {}, provider has {:?}",
        c.get::<Database>("shop", "orders-db")
            .await
            .unwrap()
            .is_some(),
        c.get::<Secret>("shop", "orders-db-credentials")
            .await
            .unwrap()
            .is_some(),
        provider.databases()
    );

    c.delete::<App>("shop", "web").await.unwrap();
    println!(
        "kubectl delete app/web: Deployments left = {}",
        c.count::<Deployment>()
    );
    println!("\n{}", ctx.metrics.render());
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("demo") | Some("all") => demo().await,
        Some("crds") => print!("{}", crds_yaml()),
        Some("run") => match kube::Client::try_default().await {
            Ok(client) => m22_kubernetes_operators_solution::controller::run(client).await,
            Err(e) => eprintln!(
                "no cluster: {e}\n(is there a current context in ~/.kube/config? see GETTING_STARTED.md)"
            ),
        },
        _ => {
            println!("22-kubernetes-operators -- reference solution\n");
            println!(
                "  cargo run -p m22-kubernetes-operators-solution -- demo   the operator on an in-memory cluster"
            );
            println!(
                "  cargo run -p m22-kubernetes-operators-solution -- crds   print the CRDs as YAML"
            );
            println!(
                "  cargo run -p m22-kubernetes-operators-solution -- run    run against your current kube context"
            );
        }
    }
}
