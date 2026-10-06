//! Exercise 3: inter-process communication -- pipes and Unix sockets.
//!
//! Processes don't share memory by default; the kernel moves bytes between
//! them. A **pipe** is a one-way byte channel (what `|` in a shell is); a
//! **Unix domain socket** is a two-way, connection-oriented channel named by
//! a path in the filesystem (Docker's `/var/run/docker.sock`, PostgreSQL's
//! local connections). Both are byte streams, so messages need framing.

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Send lines to a child `sort` through its stdin pipe and read the sorted
/// result from its stdout pipe.
pub fn sort_via_child(lines: &[&str]) -> io::Result<Vec<String>> {
    todo!("Exercise 3")
}

/// Frames larger than this are refused.
pub const MAX_MESSAGE: usize = 1 << 20;

/// `[u32 BE length][bytes]`
pub fn write_message(writer: &mut impl Write, message: &[u8]) -> io::Result<()> {
    todo!("Exercise 3")
}

/// `Ok(None)` on a clean close between messages.
pub fn read_message(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    todo!("Exercise 3")
}

/// The key-value protocol: `SET key value`, `GET key`, `DEL key` ->
/// `OK`, `VALUE value`, `NOT_FOUND`, or `ERROR reason`.
pub fn handle_command(store: &Mutex<HashMap<String, String>>, command: &str) -> String {
    todo!("Exercise 3")
}

/// Serve the key-value store on a Unix socket, a thread per client. Clients
/// share one store.
pub fn serve_kv(listener: UnixListener) {
    todo!("Exercise 3")
}

/// Bind `path` and serve on a background thread.
pub fn spawn_kv_server(path: &Path) -> io::Result<()> {
    let listener = UnixListener::bind(path)?;
    thread::spawn(move || serve_kv(listener));
    Ok(())
}

pub struct KvClient {
    stream: UnixStream,
}

impl KvClient {
    pub fn connect(path: &Path) -> io::Result<Self> {
        Ok(KvClient {
            stream: UnixStream::connect(path)?,
        })
    }

    /// Send one command, read the reply.
    pub fn request(&mut self, command: &str) -> io::Result<String> {
        write_message(&mut self.stream, command.as_bytes())?;
        let reply = read_message(&mut self.stream)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "server closed"))?;
        Ok(String::from_utf8_lossy(&reply).into_owned())
    }

    pub fn set(&mut self, key: &str, value: &str) -> io::Result<()> {
        match self.request(&format!("SET {key} {value}"))?.as_str() {
            "OK" => Ok(()),
            other => Err(io::Error::other(other.to_string())),
        }
    }

    pub fn get(&mut self, key: &str) -> io::Result<Option<String>> {
        let reply = self.request(&format!("GET {key}"))?;
        Ok(reply.strip_prefix("VALUE ").map(str::to_string))
    }

    pub fn delete(&mut self, key: &str) -> io::Result<bool> {
        Ok(self.request(&format!("DEL {key}"))? == "OK")
    }
}

/// A connected pair of sockets (`socketpair(2)`): no path, typically shared
/// between a parent and a child it forks. Here: a thread doubles numbers.
pub fn double_via_socketpair(numbers: &[u32]) -> io::Result<Vec<u32>> {
    todo!("Exercise 3")
}
