// 25-service-mesh -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m25-service-mesh -- demo     # TCP proxy, L7 sidecar, canary, mTLS, policies

use m25_service_mesh::manifests;
use m25_service_mesh::mtls::{Pki, call, serve};
use m25_service_mesh::policy::{Policy, Rule};
use m25_service_mesh::proxy::{ClusterConfig, Proxy, ProxyConfig, spawn_proxy};
use m25_service_mesh::resilience::RetryBudget;
use m25_service_mesh::routing::{Route, RouteTable, Target};
use m25_service_mesh::split::{CanaryController, Decision};
use m25_service_mesh::tcp_proxy::{TcpStats, run_tcp_proxy};
use m25_service_mesh::upstream::{Behaviour, spawn_upstream};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn cluster(endpoints: Vec<String>, timeout_ms: u64) -> ClusterConfig {
    ClusterConfig {
        endpoints,
        timeout: Duration::from_millis(timeout_ms),
    }
}

async fn demo() {
    // 1. L4: a TCP proxy in front of an HTTP server -- it doesn't care what the bytes are.
    let (web, _) = spawn_upstream("web", Behaviour::Healthy).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tcp_addr = listener.local_addr().unwrap();
    let stats = Arc::new(TcpStats::default());
    tokio::spawn(run_tcp_proxy(
        listener,
        web.trim_start_matches("http://").to_string(),
        stats.clone(),
    ));
    let mut conn = tokio::net::TcpStream::connect(tcp_addr).await.unwrap();
    conn.write_all(b"GET /through-tcp HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut reply = String::new();
    conn.read_to_string(&mut reply).await.unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await; // the proxy records the bytes after both sides close
    println!(
        "L4 proxy: {} | {} connection(s), {} bytes up, {} down\n",
        reply.lines().next().unwrap(),
        stats.connections.load(Ordering::Relaxed),
        stats.bytes_to_upstream.load(Ordering::Relaxed),
        stats.bytes_to_client.load(Ordering::Relaxed)
    );

    // 2-4, 7. The L7 sidecar.
    let (v1, _) = spawn_upstream("reviews-v1", Behaviour::Healthy).await;
    let (v2, _) = spawn_upstream("reviews-v2", Behaviour::Healthy).await;
    let (flaky, flaky_hits) = spawn_upstream("ratings-flaky", Behaviour::FailFirst(2)).await;
    let (slow, _) =
        spawn_upstream("search-slow", Behaviour::Slow(Duration::from_millis(300))).await;
    let config = ProxyConfig {
        routes: RouteTable {
            routes: vec![
                Route {
                    name: "reviews".into(),
                    path_prefix: "/reviews".into(),
                    header: None,
                    target: Target::Split(vec![
                        ("reviews-v1".into(), 80),
                        ("reviews-v2".into(), 20),
                    ]),
                },
                Route {
                    name: "reviews-canary-testers".into(),
                    path_prefix: "/reviews".into(),
                    header: Some(("x-canary".into(), "always".into())),
                    target: Target::Cluster("reviews-v2".into()),
                },
                Route {
                    name: "ratings".into(),
                    path_prefix: "/ratings".into(),
                    header: None,
                    target: Target::Cluster("ratings".into()),
                },
                Route {
                    name: "search".into(),
                    path_prefix: "/search".into(),
                    header: None,
                    target: Target::Cluster("search".into()),
                },
            ],
        },
        clusters: BTreeMap::from([
            ("reviews-v1".into(), cluster(vec![v1], 1000)),
            ("reviews-v2".into(), cluster(vec![v2], 1000)),
            ("ratings".into(), cluster(vec![flaky], 1000)),
            ("search".into(), cluster(vec![slow], 100)),
        ]),
        max_retries: 2,
        sticky_header: "x-user-id".into(),
    };
    let proxy = Arc::new(Proxy::new(
        config,
        RetryBudget::new(0.2, 10, Duration::from_secs(10)),
    ));
    let url = spawn_proxy(proxy.clone()).await;
    let client = reqwest::Client::new();
    let get = |path: &str, headers: &[(&str, &str)]| {
        let mut r = client.get(format!("{url}{path}"));
        for (k, v) in headers {
            r = r.header(*k, *v);
        }
        r.send()
    };
    let r = get(
        "/reviews/1",
        &[
            ("x-user-id", "alice"),
            ("connection", "keep-alive, x-secret-hop"),
            ("x-secret-hop", "drop me"),
        ],
    )
    .await
    .unwrap();
    let cluster_hdr = r.headers()["x-mesh-cluster"].to_str().unwrap().to_string();
    println!(
        "GET /reviews/1 as alice -> cluster {cluster_hdr}; upstream saw {}",
        r.text().await.unwrap()
    );
    let mut counts = BTreeMap::new();
    for i in 0..1000 {
        let r = get("/reviews/1", &[("x-user-id", &format!("user-{i}"))])
            .await
            .unwrap();
        *counts
            .entry(r.headers()["x-mesh-cluster"].to_str().unwrap().to_string())
            .or_insert(0) += 1;
    }
    println!("1000 users split 80/20 -> {counts:?}");
    let r = get("/reviews/1", &[("x-canary", "always")]).await.unwrap();
    println!(
        "x-canary: always -> {}",
        r.headers()["x-mesh-cluster"].to_str().unwrap()
    );
    let r = get("/ratings/5", &[]).await.unwrap();
    println!(
        "ratings (fails twice) -> {} after {} attempts ({} upstream hits)",
        r.status(),
        r.headers()["x-mesh-attempts"].to_str().unwrap(),
        flaky_hits.load(Ordering::SeqCst)
    );
    let r = get("/search?q=rust", &[]).await.unwrap();
    println!(
        "search (300 ms, timeout 100 ms) -> {} {}",
        r.status(),
        r.text().await.unwrap().trim()
    );
    println!(
        "no route -> {}\n",
        get("/nothing", &[]).await.unwrap().status()
    );

    // Progressive delivery.
    let mut canary = CanaryController::new(vec![5, 25, 50, 100], 0.01, 100);
    print!("canary steps:");
    for (requests, errors) in [(50, 0), (200, 1), (400, 2), (800, 3), (900, 1)] {
        let d = canary.evaluate(requests, errors);
        print!(" {d:?}");
        if d == Decision::Complete {
            break;
        }
    }
    let mut bad = CanaryController::new(vec![5, 25, 50, 100], 0.01, 100);
    println!(
        "\nbad canary: {:?} then {:?}, weight {}\n",
        bad.evaluate(200, 0),
        bad.evaluate(150, 9),
        bad.weight()
    );

    // 5-6. mTLS and authorization.
    let pki = Pki::new();
    let server = pki.issue(
        "spiffe://cluster.local/ns/shop/sa/inventory",
        Some("inventory.shop.svc.cluster.local"),
        false,
    );
    let orders = pki.issue("spiffe://cluster.local/ns/shop/sa/orders", None, false);
    let intruder = Pki::new().issue("spiffe://cluster.local/ns/shop/sa/orders", None, false);
    let reports = pki.issue(
        "spiffe://cluster.local/ns/analytics/sa/reports",
        None,
        false,
    );
    let policy = Policy {
        rules: vec![
            Rule {
                from: "spiffe://cluster.local/ns/shop/*".into(),
                methods: vec![],
                path_prefix: "/".into(),
            },
            Rule {
                from: "spiffe://cluster.local/ns/analytics/sa/reports".into(),
                methods: vec!["GET".into()],
                path_prefix: "/stock".into(),
            },
        ],
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(serve(
        listener,
        pki.server_config(&server),
        Arc::new(policy.clone()),
    ));
    let name = "inventory.shop.svc.cluster.local";
    println!(
        "orders   POST /reserve -> {:?}",
        call(
            &addr,
            pki.client_config(Some(&orders)),
            name,
            "POST",
            "/reserve"
        )
        .await
    );
    println!(
        "reports  GET  /stock   -> {:?}",
        call(
            &addr,
            pki.client_config(Some(&reports)),
            name,
            "GET",
            "/stock"
        )
        .await
    );
    println!(
        "reports  POST /reserve -> {:?}",
        call(
            &addr,
            pki.client_config(Some(&reports)),
            name,
            "POST",
            "/reserve"
        )
        .await
    );
    println!(
        "no cert             -> {:?}",
        call(&addr, pki.client_config(None), name, "GET", "/stock")
            .await
            .map_err(|e| e.to_string())
    );
    println!(
        "cert from another CA -> {:?}",
        call(
            &addr,
            pki.client_config(Some(&intruder)),
            name,
            "GET",
            "/stock"
        )
        .await
        .map_err(|e| e.to_string())
    );

    // Bonus.
    println!(
        "\n{}",
        serde_json::to_string_pretty(&manifests::http_route(
            "shop",
            "reviews",
            "reviews-v1",
            "reviews-v2",
            20
        ))
        .unwrap()
    );
}

#[tokio::main]
async fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("demo") | Some("all") => demo().await,
        _ => {
            println!("25-service-mesh -- your workspace\n");
            println!("  cargo run -p m25-service-mesh -- demo     every part, on local ports");
        }
    }
}
