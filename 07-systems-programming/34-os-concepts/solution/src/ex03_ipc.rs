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
    let mut child = Command::new("sort")
        .env("LC_ALL", "C")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        // Write, then drop stdin: closing the pipe is how `sort` learns the input ended.
        let mut stdin = child.stdin.take().expect("piped");
        for line in lines {
            writeln!(stdin, "{line}")?;
        }
    }
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// Frames larger than this are refused.
pub const MAX_MESSAGE: usize = 1 << 20;

/// `[u32 BE length][bytes]`
pub fn write_message(writer: &mut impl Write, message: &[u8]) -> io::Result<()> {
    writer.write_all(&(message.len() as u32).to_be_bytes())?;
    writer.write_all(message)?;
    writer.flush()
}

/// `Ok(None)` on a clean close between messages.
pub fn read_message(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut len = [0u8; 4];
    match reader.read_exact(&mut len) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let len = u32::from_be_bytes(len) as usize;
    if len > MAX_MESSAGE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "message too large",
        ));
    }
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf)?;
    Ok(Some(buf))
}

/// The key-value protocol: `SET key value`, `GET key`, `DEL key` ->
/// `OK`, `VALUE value`, `NOT_FOUND`, or `ERROR reason`.
pub fn handle_command(store: &Mutex<HashMap<String, String>>, command: &str) -> String {
    let mut parts = command.splitn(3, ' ');
    let mut store = store.lock().expect("lock");
    match (parts.next(), parts.next(), parts.next()) {
        (Some("SET"), Some(key), Some(value)) => {
            store.insert(key.into(), value.into());
            "OK".into()
        }
        (Some("GET"), Some(key), None) => store
            .get(key)
            .map_or("NOT_FOUND".into(), |v| format!("VALUE {v}")),
        (Some("DEL"), Some(key), None) => {
            if store.remove(key).is_some() {
                "OK".into()
            } else {
                "NOT_FOUND".into()
            }
        }
        _ => format!("ERROR unknown command: {command}"),
    }
}

/// Serve the key-value store on a Unix socket, a thread per client. Clients
/// share one store.
pub fn serve_kv(listener: UnixListener) {
    let store = Arc::new(Mutex::new(HashMap::new()));
    for stream in listener.incoming().flatten() {
        let store = store.clone();
        thread::spawn(move || {
            let mut stream = stream;
            while let Ok(Some(message)) = read_message(&mut stream) {
                let reply = handle_command(&store, &String::from_utf8_lossy(&message));
                if write_message(&mut stream, reply.as_bytes()).is_err() {
                    break;
                }
            }
        });
    }
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
    let (mut ours, mut theirs) = UnixStream::pair()?;
    let worker = thread::spawn(move || -> io::Result<()> {
        while let Some(message) = read_message(&mut theirs)? {
            let n = u32::from_be_bytes(
                message
                    .try_into()
                    .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
            );
            write_message(&mut theirs, &(n * 2).to_be_bytes())?;
        }
        Ok(())
    });
    let mut results = Vec::new();
    for n in numbers {
        write_message(&mut ours, &n.to_be_bytes())?;
        let reply = read_message(&mut ours)?.ok_or(io::ErrorKind::UnexpectedEof)?;
        results.push(u32::from_be_bytes(
            reply
                .try_into()
                .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
        ));
    }
    drop(ours);
    worker.join().expect("worker")?;
    Ok(results)
}
