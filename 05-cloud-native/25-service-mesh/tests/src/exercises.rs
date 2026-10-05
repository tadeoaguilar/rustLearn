use crate::sut::*;
use axum::http::{HeaderMap, HeaderValue, Method};
use balancer::Balancer;
use mtls::{Pki, call, serve, spiffe_id};
use policy::{Decision, Policy, Rule};
use proxy::{ClusterConfig, Proxy, ProxyConfig, spawn_proxy};
use resilience::{Outcome, RetryBudget, is_retryable};
use routing::{Route, RouteTable, Target, strip_hop_by_hop, upstream_headers};
use split::{CanaryController, pick_weighted};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use upstream::{Behaviour, spawn_upstream};

// ---- Exercise 1: TCP proxy -------------------------------------------------------------------------

#[tokio::test]
async fn ex1_tcp_proxy_copies_both_ways_and_counts() {
    // A raw TCP echo server.
    let echo = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let echo_addr = echo.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        while let Ok((mut s, _)) = echo.accept().await {
            tokio::spawn(async move {
                let (mut r, mut w) = s.split();
                let _ = tokio::io::copy(&mut r, &mut w).await;
            });
        }
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let stats = Arc::new(tcp_proxy::TcpStats::default());
    tokio::spawn(tcp_proxy::run_tcp_proxy(listener, echo_addr, stats.clone()));

    for _ in 0..2 {
        let mut c = tokio::net::TcpStream::connect(addr).await.unwrap();
        c.write_all(b"ping-pong").await.unwrap();
        c.shutdown().await.unwrap();
        let mut back = Vec::new();
        c.read_to_end(&mut back).await.unwrap();
        assert_eq!(back, b"ping-pong");
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(stats.connections.load(Ordering::SeqCst), 2);
    assert_eq!(stats.bytes_to_upstream.load(Ordering::SeqCst), 18);
    assert_eq!(stats.bytes_to_client.load(Ordering::SeqCst), 18);
}

#[tokio::test]
async fn ex1_unreachable_upstream_closes_the_client() {
    let dead = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_addr = dead.local_addr().unwrap().to_string();
    drop(dead);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let stats = Arc::new(tcp_proxy::TcpStats::default());
    tokio::spawn(tcp_proxy::run_tcp_proxy(listener, dead_addr, stats.clone()));
    let mut c = tokio::net::TcpStream::connect(addr).await.unwrap();
    let mut buf = Vec::new();
    let n = tokio::time::timeout(Duration::from_secs(2), c.read_to_end(&mut buf))
        .await
        .expect("closed promptly");
    assert!(n.map(|n| n == 0).unwrap_or(true));
    assert_eq!(stats.upstream_failures.load(Ordering::SeqCst), 1);
}

// ---- Exercise 2: routing and headers --------------------------------------------------------------

fn route(name: &str, prefix: &str, header: Option<(&str, &str)>) -> Route {
    Route {
        name: name.into(),
        path_prefix: prefix.into(),
        header: header.map(|(k, v)| (k.into(), v.into())),
        target: Target::Cluster(name.into()),
    }
}

#[test]
fn ex2_most_specific_route_wins() {
    let table = RouteTable {
        routes: vec![
            route("root", "/", None),
            route("api", "/api", None),
            route("api-v2", "/api/v2", None),
            route("beta", "/api", Some(("x-beta", "1"))),
        ],
    };
    let none = HeaderMap::new();
    let name = |path: &str, h: &HeaderMap| table.route(path, h).map(|r| r.name.clone());
    assert_eq!(name("/api/users", &none).as_deref(), Some("api"));
    assert_eq!(name("/api", &none).as_deref(), Some("api"));
    assert_eq!(name("/api/v2/users", &none).as_deref(), Some("api-v2"));
    assert_eq!(
        name("/apiary", &none).as_deref(),
        Some("root"),
        "prefixes match whole segments"
    );
    let mut beta = HeaderMap::new();
    beta.insert("x-beta", HeaderValue::from_static("1"));
    assert_eq!(
        name("/api/users", &beta).as_deref(),
        Some("beta"),
        "a header match beats a plain one"
    );
    assert_eq!(
        RouteTable {
            routes: vec![route("api", "/api", None)]
        }
        .route("/other", &none),
        None
    );
}

#[test]
fn ex2_hop_by_hop_and_forwarding_headers() {
    let mut h = HeaderMap::new();
    h.insert(
        "connection",
        HeaderValue::from_static("keep-alive, X-Secret-Hop"),
    );
    h.insert("keep-alive", HeaderValue::from_static("timeout=5"));
    h.insert("x-secret-hop", HeaderValue::from_static("drop me"));
    h.insert("transfer-encoding", HeaderValue::from_static("chunked"));
    h.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.7"));
    h.insert("authorization", HeaderValue::from_static("Bearer t"));
    h.insert("host", HeaderValue::from_static("proxy:8080"));
    let out = upstream_headers(&h, "10.0.0.5");
    for gone in [
        "connection",
        "keep-alive",
        "x-secret-hop",
        "transfer-encoding",
        "host",
    ] {
        assert!(!out.contains_key(gone), "{gone} must not be forwarded");
    }
    assert_eq!(out["x-forwarded-for"], "203.0.113.7, 10.0.0.5");
    assert_eq!(out["authorization"], "Bearer t", "end-to-end headers stay");
    let id = out["x-request-id"].to_str().unwrap().to_string();
    assert!(!id.is_empty());
    let mut with_id = HeaderMap::new();
    with_id.insert("x-request-id", HeaderValue::from_static("keep-me"));
    assert_eq!(
        upstream_headers(&with_id, "1.2.3.4")["x-request-id"],
        "keep-me"
    );
    let mut h2 = HeaderMap::new();
    h2.insert("upgrade", HeaderValue::from_static("websocket"));
    strip_hop_by_hop(&mut h2);
    assert!(h2.is_empty());
}

// ---- Exercise 3: retries and budgets ----------------------------------------------------------------

#[test]
fn ex3_what_is_retryable() {
    assert!(is_retryable(&Method::GET, Outcome::Status(503)));
    assert!(is_retryable(&Method::PUT, Outcome::ConnectFailed));
    assert!(
        !is_retryable(&Method::POST, Outcome::Status(503)),
        "POST isn't idempotent"
    );
    assert!(
        !is_retryable(&Method::GET, Outcome::Status(500)),
        "500 is a bug, not a blip"
    );
    assert!(!is_retryable(&Method::GET, Outcome::Status(404)));
    assert!(
        !is_retryable(&Method::GET, Outcome::TimedOut),
        "it may still be running upstream"
    );
}

#[test]
fn ex3_retry_budget() {
    let t0 = Instant::now();
    let mut b = RetryBudget::new(0.2, 2, Duration::from_secs(10));
    assert!(
        b.try_retry(t0) && b.try_retry(t0),
        "the minimum allows a couple of retries with no traffic"
    );
    assert!(!b.try_retry(t0));
    for _ in 0..10 {
        b.record_request(t0);
    }
    assert!(
        b.try_retry(t0) && b.try_retry(t0),
        "20% of 10 requests = 2 more"
    );
    assert!(!b.try_retry(t0), "budget exhausted");
    let later = t0 + Duration::from_secs(11);
    assert!(
        b.try_retry(later),
        "old requests and retries age out of the window"
    );
}

// ---- Exercise 4: splits and canaries --------------------------------------------------------------

#[test]
fn ex4_weighted_and_sticky() {
    let w = vec![("v1".to_string(), 80), ("v2".to_string(), 20)];
    let mut v2 = 0;
    for roll in 0..1000u64 {
        if pick_weighted(&w, None, roll) == Some("v2") {
            v2 += 1;
        }
    }
    assert_eq!(v2, 200, "exactly proportional over a full cycle of rolls");
    for user in ["alice", "bob", "carol"] {
        let first = pick_weighted(&w, Some(user), 0);
        assert!(
            (1..50).all(|roll| pick_weighted(&w, Some(user), roll) == first),
            "sticky per user"
        );
    }
    let users: Vec<String> = (0..2000).map(|i| format!("user-{i}")).collect();
    let on_v2 = users
        .iter()
        .filter(|u| pick_weighted(&w, Some(u.as_str()), 0) == Some("v2"))
        .count();
    assert!((300..500).contains(&on_v2), "about 20% of users: {on_v2}");
    assert_eq!(
        pick_weighted(&[("a".into(), 0), ("b".into(), 5)], None, 3),
        Some("b")
    );
    assert_eq!(pick_weighted(&[("a".into(), 0)], None, 3), None);
    assert_eq!(split::fnv1a("a"), 0xaf63dc4c8601ec8c, "standard FNV-1a");
}

#[test]
fn ex4_canary_controller() {
    use split::Decision::*;
    let mut c = CanaryController::new(vec![5, 25, 50, 100], 0.01, 100);
    assert_eq!(c.weight(), 5);
    assert_eq!(
        c.weights("stable", "canary"),
        vec![("stable".to_string(), 95), ("canary".to_string(), 5)]
    );
    assert_eq!(c.evaluate(40, 0), Hold, "not enough traffic to judge");
    assert_eq!(c.evaluate(200, 1), Promote(25));
    assert_eq!(c.evaluate(500, 4), Promote(50));
    assert_eq!(c.evaluate(500, 0), Complete);
    assert_eq!(c.weight(), 100);

    let mut bad = CanaryController::new(vec![5, 25, 50, 100], 0.01, 100);
    assert_eq!(
        bad.evaluate(20, 5),
        Rollback,
        "a clearly bad canary is rolled back without waiting"
    );
    assert_eq!(bad.weight(), 0);
    assert_eq!(bad.evaluate(1000, 0), Rollback, "and stays rolled back");
}

// ---- Exercise 5: mTLS -------------------------------------------------------------------------------

const INVENTORY: &str = "spiffe://cluster.local/ns/shop/sa/inventory";
const ORDERS: &str = "spiffe://cluster.local/ns/shop/sa/orders";
const HOST: &str = "inventory.shop.svc.cluster.local";

async fn mtls_server(pki: &Pki, policy: Policy) -> String {
    let id = pki.issue(INVENTORY, Some(HOST), false);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(serve(listener, pki.server_config(&id), Arc::new(policy)));
    addr
}

fn allow_shop() -> Policy {
    Policy {
        rules: vec![Rule {
            from: "spiffe://cluster.local/ns/shop/*".into(),
            methods: vec![],
            path_prefix: "/".into(),
        }],
    }
}

#[test]
fn ex5_certificates_carry_spiffe_ids() {
    let pki = Pki::new();
    let id = pki.issue(ORDERS, None, false);
    assert_eq!(spiffe_id(&id.cert).as_deref(), Some(ORDERS));
    assert_eq!(id.spiffe_id, ORDERS);
    assert_eq!(
        spiffe_id(&pki.ca_cert()),
        None,
        "the CA has no workload identity"
    );
}

#[tokio::test]
async fn ex5_mutual_tls() {
    let pki = Pki::new();
    let addr = mtls_server(&pki, allow_shop()).await;
    let orders = pki.issue(ORDERS, None, false);
    let reply = call(
        &addr,
        pki.client_config(Some(&orders)),
        HOST,
        "GET",
        "/stock",
    )
    .await
    .unwrap();
    assert_eq!(
        reply,
        format!("200 hello {ORDERS}"),
        "the server sees the client's verified identity"
    );

    assert!(
        call(&addr, pki.client_config(None), HOST, "GET", "/stock")
            .await
            .is_err(),
        "no client certificate"
    );
    let foreign = Pki::new();
    assert!(
        call(
            &addr,
            pki.client_config(Some(&foreign.issue(ORDERS, None, false))),
            HOST,
            "GET",
            "/"
        )
        .await
        .is_err(),
        "same id, another CA"
    );
    assert!(
        call(
            &addr,
            pki.client_config(Some(&pki.issue(ORDERS, None, true))),
            HOST,
            "GET",
            "/"
        )
        .await
        .is_err(),
        "expired certificate"
    );
    assert!(
        call(
            &addr,
            pki.client_config(Some(&orders)),
            "payments.shop.svc.cluster.local",
            "GET",
            "/"
        )
        .await
        .is_err(),
        "the client checks the server's name too"
    );
    assert!(
        call(
            &addr,
            foreign.client_config(Some(&foreign.issue(ORDERS, None, false))),
            HOST,
            "GET",
            "/"
        )
        .await
        .is_err(),
        "a client that doesn't trust our CA"
    );
}

// ---- Exercise 6: authorization ----------------------------------------------------------------------

#[test]
fn ex6_policy() {
    let p = Policy {
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
    assert_eq!(
        p.authorize(Some(ORDERS), "POST", "/reserve"),
        Decision::Allow
    );
    let reports = Some("spiffe://cluster.local/ns/analytics/sa/reports");
    assert_eq!(
        p.authorize(reports, "get", "/stock/BOOK-1"),
        Decision::Allow,
        "methods are case-insensitive"
    );
    assert_eq!(p.authorize(reports, "POST", "/stock"), Decision::Deny);
    assert_eq!(p.authorize(reports, "GET", "/admin"), Decision::Deny);
    assert_eq!(
        p.authorize(Some("spiffe://cluster.local/ns/shopping/sa/x"), "GET", "/"),
        Decision::Deny,
        "ns/shop/* covers the shop namespace, not shopping"
    );
    assert_eq!(p.authorize(None, "GET", "/"), Decision::Unauthenticated);
    assert_eq!(
        Policy::default().authorize(Some(ORDERS), "GET", "/"),
        Decision::Deny,
        "deny by default"
    );
}

#[tokio::test]
async fn ex6_enforced_over_mtls() {
    let pki = Pki::new();
    let policy = Policy {
        rules: vec![Rule {
            from: ORDERS.into(),
            methods: vec!["POST".into()],
            path_prefix: "/reserve".into(),
        }],
    };
    let addr = mtls_server(&pki, policy).await;
    let orders = pki.client_config(Some(&pki.issue(ORDERS, None, false)));
    assert!(
        call(&addr, orders.clone(), HOST, "POST", "/reserve")
            .await
            .unwrap()
            .starts_with("200")
    );
    assert!(
        call(&addr, orders, HOST, "DELETE", "/reserve")
            .await
            .unwrap()
            .starts_with("403")
    );
    let other = pki.client_config(Some(&pki.issue(
        "spiffe://cluster.local/ns/shop/sa/web",
        None,
        false,
    )));
    assert!(
        call(&addr, other, HOST, "POST", "/reserve")
            .await
            .unwrap()
            .starts_with("403")
    );
}

// ---- Exercise 7: load balancing and outliers ------------------------------------------------------

fn urls(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("http://e{i}")).collect()
}

#[test]
fn ex7_p2c_prefers_the_less_loaded() {
    let t = Instant::now();
    let mut lb = Balancer::new(&urls(3), 3, Duration::from_secs(30));
    assert_eq!(lb.pick(t, 0, 0).unwrap().1, "http://e0");
    assert_eq!(lb.pick(t, 0, 1).unwrap().1, "http://e1", "e0 is busier");
    assert_eq!(
        lb.pick(t, 0, 1).unwrap().1,
        "http://e0",
        "tie: the first choice"
    );
    assert_eq!(
        (lb.in_flight("http://e0"), lb.in_flight("http://e1")),
        (2, 1)
    );
    lb.finish(0, true, t);
    assert_eq!(lb.in_flight("http://e0"), 1);
}

#[test]
fn ex7_outlier_ejection_with_a_cap() {
    let t = Instant::now();
    let mut lb = Balancer::new(&urls(4), 3, Duration::from_secs(30));
    let fail = |lb: &mut Balancer, i: usize, now| {
        let (idx, _) = lb.pick(now, i, i).unwrap();
        lb.finish(idx, false, now);
    };
    for _ in 0..3 {
        fail(&mut lb, 0, t);
    }
    assert!(lb.is_ejected("http://e0", t), "3 consecutive failures");
    for _ in 0..50 {
        let (i, url) = lb.pick(t, 0, 1).unwrap();
        assert_ne!(url, "http://e0", "ejected endpoints get no traffic");
        lb.finish(i, true, t);
    }
    // e1 fails too: 2 of 4 ejected = the 50% cap.
    for _ in 0..3 {
        let (i, _) = (0..)
            .map(|k| lb.pick(t, k, k).unwrap())
            .find(|(_, u)| u == "http://e1")
            .unwrap();
        lb.finish(i, false, t);
    }
    assert!(lb.is_ejected("http://e1", t));
    for _ in 0..3 {
        let (i, _) = (0..)
            .map(|k| lb.pick(t, k, k).unwrap())
            .find(|(_, u)| u == "http://e2")
            .unwrap();
        lb.finish(i, false, t);
    }
    assert!(!lb.is_ejected("http://e2", t), "never eject more than half");
    assert!(
        !lb.is_ejected("http://e0", t + Duration::from_secs(31)),
        "ejection ends"
    );
}

// ---- The sidecar: Exercises 2-4 and 7 together -------------------------------------------------------

async fn sidecar(
    routes: Vec<Route>,
    clusters: Vec<(&str, Vec<String>, u64)>,
) -> (Arc<Proxy>, String) {
    let config = ProxyConfig {
        routes: RouteTable { routes },
        clusters: clusters
            .into_iter()
            .map(|(n, eps, ms)| {
                (
                    n.to_string(),
                    ClusterConfig {
                        endpoints: eps,
                        timeout: Duration::from_millis(ms),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
        max_retries: 2,
        sticky_header: "x-user-id".into(),
    };
    let proxy = Arc::new(Proxy::new(
        config,
        RetryBudget::new(0.2, 10, Duration::from_secs(10)),
    ));
    let url = spawn_proxy(proxy.clone()).await;
    (proxy, url)
}

#[tokio::test]
async fn sidecar_forwards_with_headers_and_splits() {
    let (v1, _) = spawn_upstream("v1", Behaviour::Healthy).await;
    let (v2, _) = spawn_upstream("v2", Behaviour::Healthy).await;
    let split = Route {
        name: "reviews".into(),
        path_prefix: "/reviews".into(),
        header: None,
        target: Target::Split(vec![("v1".into(), 50), ("v2".into(), 50)]),
    };
    let (_p, url) = sidecar(
        vec![split],
        vec![("v1", vec![v1], 1000), ("v2", vec![v2], 1000)],
    )
    .await;
    let client = reqwest::Client::new();
    let r = client
        .get(format!("{url}/reviews/7?full=1"))
        .header("x-user-id", "alice")
        .header("connection", "x-secret-hop")
        .header("x-secret-hop", "x")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let cluster = r.headers()["x-mesh-cluster"].to_str().unwrap().to_string();
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["upstream"], cluster.as_str());
    assert_eq!(body["path"], "/reviews/7?full=1");
    assert_eq!(body["x-forwarded-for"], "127.0.0.1");
    assert!(body["x-request-id"].is_string());
    assert!(body["x-secret-hop"].is_null());
    for _ in 0..10 {
        let again = client
            .get(format!("{url}/reviews/7"))
            .header("x-user-id", "alice")
            .send()
            .await
            .unwrap();
        assert_eq!(
            again.headers()["x-mesh-cluster"],
            cluster.as_str(),
            "sticky"
        );
    }
    assert_eq!(
        client
            .get(format!("{url}/nothing"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
}

#[tokio::test]
async fn sidecar_retries_times_out_and_spreads_load() {
    let (flaky, flaky_hits) = spawn_upstream("flaky", Behaviour::FailFirst(2)).await;
    let (slow, _) = spawn_upstream("slow", Behaviour::Slow(Duration::from_millis(500))).await;
    let (a, a_hits) = spawn_upstream("a", Behaviour::Healthy).await;
    let (b, b_hits) = spawn_upstream("b", Behaviour::Healthy).await;
    let (broken, broken_hits) = spawn_upstream("broken", Behaviour::AlwaysFail(503)).await;
    let routes = vec![
        route("flaky", "/flaky", None),
        route("slow", "/slow", None),
        route("pool", "/pool", None),
        route("broken", "/broken", None),
    ];
    let (proxy, url) = sidecar(
        routes,
        vec![
            ("flaky", vec![flaky], 1000),
            ("slow", vec![slow], 100),
            ("pool", vec![a, b], 1000),
            ("broken", vec![broken], 1000),
        ],
    )
    .await;
    let client = reqwest::Client::new();

    let r = client.get(format!("{url}/flaky")).send().await.unwrap();
    assert_eq!(
        (
            r.status().as_u16(),
            r.headers()["x-mesh-attempts"].to_str().unwrap()
        ),
        (200, "3")
    );
    assert_eq!(flaky_hits.load(Ordering::SeqCst), 3);
    assert_eq!(
        proxy.stats("flaky"),
        proxy::ClusterStats {
            requests: 3,
            errors: 2
        }
    );

    let r = client.post(format!("{url}/broken")).send().await.unwrap();
    assert_eq!(
        (r.status().as_u16(), broken_hits.load(Ordering::SeqCst)),
        (503, 1),
        "POST is never retried"
    );

    let t = Instant::now();
    let r = client.get(format!("{url}/slow")).send().await.unwrap();
    assert_eq!(r.status(), 504);
    assert!(
        t.elapsed() < Duration::from_millis(400),
        "the cluster timeout applies, and timeouts aren't retried"
    );

    for _ in 0..40 {
        assert_eq!(
            client
                .get(format!("{url}/pool"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    let (ha, hb) = (a_hits.load(Ordering::SeqCst), b_hits.load(Ordering::SeqCst));
    assert_eq!(ha + hb, 40);
    assert!(ha >= 5 && hb >= 5, "both endpoints get traffic: {ha}/{hb}");
}

// ---- Bonus: manifests ---------------------------------------------------------------------------------

#[test]
fn bonus_mesh_manifests() {
    let route = manifests::http_route("shop", "reviews", "reviews-v1", "reviews-v2", 20);
    assert_eq!(route["kind"], "HTTPRoute");
    let backends = route["spec"]["rules"][0]["backendRefs"].as_array().unwrap();
    assert_eq!(
        (
            backends[0]["weight"].as_u64(),
            backends[1]["weight"].as_u64()
        ),
        (Some(80), Some(20))
    );

    let policy = Policy {
        rules: vec![Rule {
            from: ORDERS.into(),
            methods: vec!["POST".into()],
            path_prefix: "/reserve".into(),
        }],
    };
    let ap = manifests::authorization_policy("shop", "inventory", &policy);
    assert_eq!(ap["spec"]["action"], "ALLOW");
    let rule = &ap["spec"]["rules"][0];
    assert_eq!(
        rule["from"][0]["source"]["principals"][0],
        "cluster.local/ns/shop/sa/orders"
    );
    assert_eq!(rule["to"][0]["operation"]["methods"][0], "POST");
    assert_eq!(rule["to"][0]["operation"]["paths"][0], "/reserve*");
    assert_eq!(
        manifests::strict_mtls("shop")["spec"]["mtls"]["mode"],
        "STRICT"
    );
}
