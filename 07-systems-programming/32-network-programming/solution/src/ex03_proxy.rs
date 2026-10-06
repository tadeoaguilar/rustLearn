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
    tokio::io::copy_bidirectional(&mut client, &mut upstream).await
}

/// Proxy every connection to `upstream`, counting in `stats`.
pub async fn serve_proxy(
    listener: TcpListener,
    upstream: SocketAddr,
    stats: Arc<Stats>,
) -> io::Result<()> {
    loop {
        let (client, _) = listener.accept().await?;
        let stats = stats.clone();
        tokio::spawn(async move {
            let Ok(server) = TcpStream::connect(upstream).await else {
                return; // dropping `client` closes it
            };
            stats.connections.fetch_add(1, Ordering::SeqCst);
            if let Ok((up, down)) = pipe(client, server).await {
                stats.bytes_up.fetch_add(up, Ordering::SeqCst);
                stats.bytes_down.fetch_add(down, Ordering::SeqCst);
            }
        });
    }
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
        let healthy = upstreams.iter().map(|_| AtomicBool::new(true)).collect();
        Balancer {
            upstreams,
            healthy,
            next: AtomicUsize::new(0),
        }
    }

    pub fn set_healthy(&self, index: usize, healthy: bool) {
        self.healthy[index].store(healthy, Ordering::SeqCst);
    }

    pub fn is_healthy(&self, index: usize) -> bool {
        self.healthy[index].load(Ordering::SeqCst)
    }

    /// The next healthy upstream in round-robin order, or `None` if all
    /// are down. Each call advances the cursor, so load spreads evenly.
    pub fn pick(&self) -> Option<SocketAddr> {
        let n = self.upstreams.len();
        for _ in 0..n {
            let i = self.next.fetch_add(1, Ordering::SeqCst) % n;
            if self.is_healthy(i) {
                return Some(self.upstreams[i]);
            }
        }
        None
    }

    /// One round of health checks: an upstream is healthy if a TCP
    /// connection to it opens within `timeout`.
    pub async fn check_all(&self, timeout: Duration) {
        for (i, addr) in self.upstreams.iter().enumerate() {
            let up = matches!(
                tokio::time::timeout(timeout, TcpStream::connect(addr)).await,
                Ok(Ok(_))
            );
            self.set_healthy(i, up);
        }
    }
}

/// Check health every `interval`, forever.
pub async fn health_loop(balancer: Arc<Balancer>, interval: Duration) {
    loop {
        balancer.check_all(interval / 2).await;
        tokio::time::sleep(interval).await;
    }
}

/// Like `serve_proxy`, choosing the upstream per connection. If the chosen
/// one refuses, mark it unhealthy and try the next.
pub async fn serve_balanced(
    listener: TcpListener,
    balancer: Arc<Balancer>,
    stats: Arc<Stats>,
) -> io::Result<()> {
    loop {
        let (client, _) = listener.accept().await?;
        let (balancer, stats) = (balancer.clone(), stats.clone());
        tokio::spawn(async move {
            for _ in 0..balancer.upstreams.len() {
                let Some(addr) = balancer.pick() else { return };
                match TcpStream::connect(addr).await {
                    Ok(server) => {
                        stats.connections.fetch_add(1, Ordering::SeqCst);
                        if let Ok((up, down)) = pipe(client, server).await {
                            stats.bytes_up.fetch_add(up, Ordering::SeqCst);
                            stats.bytes_down.fetch_add(down, Ordering::SeqCst);
                        }
                        return;
                    }
                    Err(_) => {
                        if let Some(i) = balancer.upstreams.iter().position(|a| *a == addr) {
                            balancer.set_healthy(i, false);
                        }
                    }
                }
            }
        });
    }
}
