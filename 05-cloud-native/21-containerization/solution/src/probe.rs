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
    match tokio::time::timeout(timeout, exchange(addr, path)).await {
        Err(_) => Err(ProbeError::Timeout),
        Ok(result) => result,
    }
}

async fn exchange(addr: SocketAddr, path: &str) -> Result<u16, ProbeError> {
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|e| ProbeError::Connect(e.to_string()))?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|e| ProbeError::Connect(e.to_string()))?;

    // Only the status line matters: "HTTP/1.1 200 OK\r\n".
    let mut buf = Vec::new();
    let mut chunk = [0u8; 256];
    while !buf.windows(2).any(|w| w == b"\r\n") {
        let n = stream
            .read(&mut chunk)
            .await
            .map_err(|e| ProbeError::Connect(e.to_string()))?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let text = String::from_utf8_lossy(&buf);
    let line = text.lines().next().unwrap_or("");
    let status = match line.split_whitespace().collect::<Vec<_>>().as_slice() {
        [version, code, ..] if version.starts_with("HTTP/") => code
            .parse::<u16>()
            .map_err(|_| ProbeError::BadResponse(line.to_string()))?,
        _ => return Err(ProbeError::BadResponse(line.to_string())),
    };
    if (200..300).contains(&status) {
        Ok(status)
    } else {
        Err(ProbeError::Unhealthy(status))
    }
}

/// Docker reads a health check's exit code: 0 healthy, 1 unhealthy.
pub fn exit_code(result: &Result<u16, ProbeError>) -> i32 {
    if result.is_ok() { 0 } else { 1 }
}
