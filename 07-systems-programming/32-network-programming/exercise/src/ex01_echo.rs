//! Exercise 1: TCP echo servers, blocking and async.
//!
//! The same protocol twice: each line a client sends comes back prefixed
//! with `echo: `; the line `bye` closes the connection. The blocking version
//! uses one OS thread per client; the async version one tokio *task* per
//! client -- thousands of connections on a handful of threads.

use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

/// The reply to one line, or `None` to hang up.
pub fn reply(line: &str) -> Option<String> {
    todo!("Exercise 1")
}

/// Serve one client on a blocking stream until `bye` or EOF.
pub fn handle_blocking(stream: TcpStream) -> io::Result<()> {
    todo!("Exercise 1")
}

/// Accept forever, a thread per client. `served` counts finished clients.
pub fn serve_blocking(listener: TcpListener, served: Arc<AtomicUsize>) {
    todo!("Exercise 1")
}

/// Serve one client on an async stream until `bye` or EOF.
pub async fn handle_async(stream: tokio::net::TcpStream) -> io::Result<()> {
    todo!("Exercise 1")
}

/// Accept forever, a task per client.
pub async fn serve_async(listener: tokio::net::TcpListener) -> io::Result<()> {
    todo!("Exercise 1")
}

/// A client: send each line, collect each reply, then say `bye`.
pub async fn echo_client(addr: std::net::SocketAddr, lines: &[&str]) -> io::Result<Vec<String>> {
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let (read, mut write) = stream.into_split();
    let mut replies = AsyncBufReader::new(read).lines();
    let mut out = Vec::new();
    for line in lines {
        write.write_all(format!("{line}\n").as_bytes()).await?;
        out.push(replies.next_line().await?.unwrap_or_default());
    }
    write.write_all(b"bye\n").await?;
    Ok(out)
}
