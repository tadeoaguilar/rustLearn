//! Exercise 3: a TCP proxy and a load balancer.
//!
//! A proxy accepts a client, opens a connection to an upstream server and
//! copies bytes both ways until either side closes. It doesn't understand
//! the protocol -- HTTP, Redis, SSH all pass through. A load balancer is a
//! proxy that picks the upstream: round-robin over the ones a background
//! health check currently believes are up.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};

/// Bytes moved through the proxy, totalled over all connections.
#[derive(Debug, Default)]
pub struct Stats {
    pub connections: AtomicU64,
    /// Client -> upstream.
    pub bytes_up: AtomicU64,
    /// Upstream -> client.
    pub bytes_down: AtomicU64,
}

/// Copy both ways between `client` and `upstream` until both directions
/// are done; returns `(bytes up, bytes down)`.
pub async fn pipe(mut client: TcpStream, mut upstream: TcpStream) -> io::Result<(u64, u64)> {
    todo!("Exercise 3")
}

/// Proxy every connection to `upstream`, counting in `stats`.
pub async fn serve_proxy(
    listener: TcpListener,
    upstream: SocketAddr,
    stats: Arc<Stats>,
) -> io::Result<()> {
    todo!("Exercise 3")
}

/// The upstreams, their health, and the round-robin cursor.
#[derive(Debug)]
pub struct Balancer {
    pub upstreams: Vec<SocketAddr>,
    healthy: Vec<AtomicBool>,
    next: AtomicUsize,
}

impl Balancer {
    /// Every upstream starts healthy.
    pub fn new(upstreams: Vec<SocketAddr>) -> Self {
        todo!("Exercise 3")
    }

    pub fn set_healthy(&self, index: usize, healthy: bool) {
        todo!("Exercise 3")
    }

    pub fn is_healthy(&self, index: usize) -> bool {
        todo!("Exercise 3")
    }

    /// The next healthy upstream in round-robin order, or `None` if all
    /// are down. Each call advances the cursor, so load spreads evenly.
    pub fn pick(&self) -> Option<SocketAddr> {
        todo!("Exercise 3")
    }

    /// One round of health checks: an upstream is healthy if a TCP
    /// connection to it opens within `timeout`.
    pub async fn check_all(&self, timeout: Duration) {
        todo!("Exercise 3")
    }
}

/// Check health every `interval`, forever.
pub async fn health_loop(balancer: Arc<Balancer>, interval: Duration) {
    todo!("Exercise 3")
}

/// Like `serve_proxy`, choosing the upstream per connection. If the chosen
/// one refuses, mark it unhealthy and try the next.
pub async fn serve_balanced(
    listener: TcpListener,
    balancer: Arc<Balancer>,
    stats: Arc<Stats>,
) -> io::Result<()> {
    todo!("Exercise 3")
}
