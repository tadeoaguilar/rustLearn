//! Exercise 1: an L4 (TCP) proxy.
//!
//! The simplest sidecar: accept a connection, open one to the upstream, copy
//! bytes both ways until either side closes. It sees connections and bytes,
//! nothing of HTTP -- which is also why it works for any protocol (Postgres,
//! Redis, gRPC...). Linkerd and Istio fall back to this for traffic they
//! can't parse.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Default)]
pub struct TcpStats {
    pub connections: AtomicU64,
    pub bytes_to_upstream: AtomicU64,
    pub bytes_to_client: AtomicU64,
    pub upstream_failures: AtomicU64,
}

/// Proxies every connection on `listener` to `upstream`, one task each.
pub async fn run_tcp_proxy(listener: TcpListener, upstream: String, stats: Arc<TcpStats>) {
    todo!("Exercise 1")
}
