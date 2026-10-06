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
    let line = line.trim_end_matches(['\r', '\n']);
    (line != "bye").then(|| format!("echo: {line}\n"))
}

/// Serve one client on a blocking stream until `bye` or EOF.
pub fn handle_blocking(stream: TcpStream) -> io::Result<()> {
    let mut writer = stream.try_clone()?;
    for line in BufReader::new(stream).lines() {
        match reply(&line?) {
            Some(answer) => writer.write_all(answer.as_bytes())?,
            None => break,
        }
    }
    Ok(())
}

/// Accept forever, a thread per client. `served` counts finished clients.
pub fn serve_blocking(listener: TcpListener, served: Arc<AtomicUsize>) {
    for stream in listener.incoming().flatten() {
        let served = served.clone();
        thread::spawn(move || {
            let _ = handle_blocking(stream);
            served.fetch_add(1, Ordering::SeqCst);
        });
    }
}

/// Serve one client on an async stream until `bye` or EOF.
pub async fn handle_async(stream: tokio::net::TcpStream) -> io::Result<()> {
    let (read, mut write) = stream.into_split();
    let mut lines = AsyncBufReader::new(read).lines();
    while let Some(line) = lines.next_line().await? {
        match reply(&line) {
            Some(answer) => write.write_all(answer.as_bytes()).await?,
            None => break,
        }
    }
    write.shutdown().await
}

/// Accept forever, a task per client.
pub async fn serve_async(listener: tokio::net::TcpListener) -> io::Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = handle_async(stream).await;
        });
    }
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
