//! Exercises 2-4 and 7 together: the L7 sidecar proxy.
//!
//! Every request: route it, pick a cluster (weighted, sticky), pick an
//! endpoint (P2C, skipping ejected ones), forward with the cluster's
//! timeout, retry if it's safe and the budget allows, and return the answer.

use crate::balancer::Balancer;
use crate::resilience::{Outcome, RetryBudget, is_retryable};
use crate::routing::{RouteTable, Target, strip_hop_by_hop, upstream_headers};
use crate::split::pick_weighted;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use rand::Rng;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub endpoints: Vec<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub routes: RouteTable,
    pub clusters: BTreeMap<String, ClusterConfig>,
    pub max_retries: u32,
    /// The header that keeps a user on one side of a split.
    pub sticky_header: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClusterStats {
    pub requests: u64,
    pub errors: u64,
}

pub struct Proxy {
    config: ProxyConfig,
    balancers: Mutex<BTreeMap<String, Balancer>>,
    budget: Mutex<RetryBudget>,
    client: reqwest::Client,
    stats: Mutex<BTreeMap<String, ClusterStats>>,
}

impl Proxy {
    pub fn new(config: ProxyConfig, budget: RetryBudget) -> Proxy {
        let balancers = config
            .clusters
            .iter()
            .map(|(name, c)| {
                (
                    name.clone(),
                    Balancer::new(&c.endpoints, 3, Duration::from_secs(30)),
                )
            })
            .collect();
        Proxy {
            config,
            balancers: Mutex::new(balancers),
            budget: Mutex::new(budget),
            client: reqwest::Client::new(),
            stats: Mutex::default(),
        }
    }

    /// Per-cluster attempts and failures (5xx, timeouts, connection errors).
    pub fn stats(&self, cluster: &str) -> ClusterStats {
        self.stats
            .lock()
            .unwrap()
            .get(cluster)
            .copied()
            .unwrap_or_default()
    }
}

fn mesh_error(status: StatusCode, message: &str) -> Response {
    (status, [("x-mesh-error", "true")], format!("{message}\n")).into_response()
}

async fn handle(
    State(proxy): State<Arc<Proxy>>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let path = parts.uri.path().to_string();
    let path_and_query = parts
        .uri
        .path_and_query()
        .map_or(path.clone(), |p| p.as_str().to_string());

    let Some(route) = proxy.config.routes.route(&path, &parts.headers) else {
        return mesh_error(StatusCode::NOT_FOUND, "no route");
    };
    let cluster_name = match &route.target {
        Target::Cluster(c) => c.clone(),
        Target::Split(weights) => {
            let key = parts
                .headers
                .get(proxy.config.sticky_header.as_str())
                .and_then(|v| v.to_str().ok());
            match pick_weighted(weights, key, rand::thread_rng().r#gen()) {
                Some(c) => c.to_string(),
                None => {
                    return mesh_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "all split weights are zero",
                    );
                }
            }
        }
    };
    let Some(cluster) = proxy.config.clusters.get(&cluster_name) else {
        return mesh_error(StatusCode::BAD_GATEWAY, "unknown cluster");
    };
    let Ok(body) = to_bytes(body, 10 * 1024 * 1024).await else {
        return mesh_error(StatusCode::PAYLOAD_TOO_LARGE, "body too large");
    };
    let headers = upstream_headers(&parts.headers, &client.ip().to_string());
    proxy.budget.lock().unwrap().record_request(Instant::now());

    let mut attempts = 0;
    loop {
        attempts += 1;
        let picked = {
            let mut rng = rand::thread_rng();
            let (a, b) = (rng.r#gen::<usize>(), rng.r#gen::<usize>());
            proxy
                .balancers
                .lock()
                .unwrap()
                .get_mut(&cluster_name)
                .and_then(|lb| lb.pick(Instant::now(), a, b))
        };
        let Some((index, endpoint)) = picked else {
            return mesh_error(StatusCode::SERVICE_UNAVAILABLE, "no healthy endpoints");
        };
        let send = proxy
            .client
            .request(parts.method.clone(), format!("{endpoint}{path_and_query}"))
            .headers(headers.clone())
            .body(body.clone())
            .send();
        let result = tokio::time::timeout(cluster.timeout, send).await;
        let outcome = match &result {
            Err(_) => Outcome::TimedOut,
            Ok(Err(_)) => Outcome::ConnectFailed,
            Ok(Ok(r)) => Outcome::Status(r.status().as_u16()),
        };
        let failed = !matches!(outcome, Outcome::Status(s) if s < 500);
        proxy
            .balancers
            .lock()
            .unwrap()
            .get_mut(&cluster_name)
            .unwrap()
            .finish(index, !failed, Instant::now());
        {
            let mut stats = proxy.stats.lock().unwrap();
            let s = stats.entry(cluster_name.clone()).or_default();
            s.requests += 1;
            s.errors += u64::from(failed);
        }
        if attempts <= proxy.config.max_retries
            && is_retryable(&parts.method, outcome)
            && proxy.budget.lock().unwrap().try_retry(Instant::now())
        {
            continue;
        }
        let upstream = match result {
            Err(_) => return mesh_error(StatusCode::GATEWAY_TIMEOUT, "upstream timed out"),
            Ok(Err(_)) => return mesh_error(StatusCode::BAD_GATEWAY, "upstream unreachable"),
            Ok(Ok(r)) => r,
        };
        let status = upstream.status();
        let mut response_headers = upstream.headers().clone();
        strip_hop_by_hop(&mut response_headers);
        let bytes = upstream.bytes().await.unwrap_or_default();
        let mut response = Response::new(Body::from(bytes));
        *response.status_mut() = status;
        *response.headers_mut() = response_headers;
        response.headers_mut().remove("content-length"); // recomputed for the new body
        response.headers_mut().insert(
            "x-mesh-cluster",
            HeaderValue::from_str(&cluster_name).unwrap(),
        );
        response
            .headers_mut()
            .insert("x-mesh-attempts", HeaderValue::from(attempts));
        return response;
    }
}

pub fn router(proxy: Arc<Proxy>) -> Router {
    Router::new().fallback(handle).with_state(proxy)
}

/// Starts the proxy on a random port; returns "http://127.0.0.1:port".
pub async fn spawn_proxy(proxy: Arc<Proxy>) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let url = format!("http://{}", listener.local_addr().unwrap());
    let app = router(proxy).into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, app).await.expect("proxy") });
    url
}
