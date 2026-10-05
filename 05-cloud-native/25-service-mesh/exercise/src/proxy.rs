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
    todo!("Exercises 2-4 and 7: the sidecar")
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
