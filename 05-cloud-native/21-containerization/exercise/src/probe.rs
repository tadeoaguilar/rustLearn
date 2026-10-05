//! Exercise 4: a health check built into the binary.
//!
//! `HEALTHCHECK CMD curl -f http://localhost:8080/readyz` needs curl in the
//! image. A `scratch` or distroless image has no curl and no shell -- so the
//! binary checks itself: `HEALTHCHECK CMD ["/app", "healthcheck"]`.
//!
//! HTTP/1.1 is simple enough to speak over a raw TCP socket, which keeps an
//! HTTP client library out of the image.

use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeError {
    /// Nothing listening, or the connection failed.
    Connect(String),
    /// Connected, but no complete answer in time.
    Timeout,
    /// The reply wasn't HTTP.
    BadResponse(String),
    /// A non-2xx status.
    Unhealthy(u16),
}

/// GETs `path` and returns the status if it's 2xx. The whole exchange --
/// connect, write, read -- must finish within `timeout`.
pub async fn probe(addr: SocketAddr, path: &str, timeout: Duration) -> Result<u16, ProbeError> {
    todo!("Exercise 4")
}

async fn exchange(addr: SocketAddr, path: &str) -> Result<u16, ProbeError> {
    todo!("Exercise 4")
}

/// Docker reads a health check's exit code: 0 healthy, 1 unhealthy.
pub fn exit_code(result: &Result<u16, ProbeError>) -> i32 {
    todo!("Exercise 4")
}
