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
    while let Ok((mut client, _)) = listener.accept().await {
        let (upstream, stats) = (upstream.clone(), stats.clone());
        tokio::spawn(async move {
            stats.connections.fetch_add(1, Ordering::Relaxed);
            let Ok(mut server) = TcpStream::connect(&upstream).await else {
                // Nothing to talk to: close the client's connection.
                stats.upstream_failures.fetch_add(1, Ordering::Relaxed);
                return;
            };
            // Copies both directions concurrently; returns when both are done.
            if let Ok((up, down)) = tokio::io::copy_bidirectional(&mut client, &mut server).await {
                stats.bytes_to_upstream.fetch_add(up, Ordering::Relaxed);
                stats.bytes_to_client.fetch_add(down, Ordering::Relaxed);
            }
        });
    }
}
