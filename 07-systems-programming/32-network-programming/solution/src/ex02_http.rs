//! Exercise 2: an HTTP/1.1 server from scratch.
//!
//! HTTP/1.1 is text over TCP: a request line, header lines, an empty line,
//! then `Content-Length` bytes of body. The server answers in the same shape
//! and, unless asked to close, keeps the connection open for the next
//! request (keep-alive). Everything a framework does for you is here in
//! miniature: parsing with size limits, percent-decoding, routing, static
//! files without path traversal, and the right status codes for bad input.

use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// Request line plus headers may not exceed this.
pub const MAX_HEAD: usize = 8 * 1024;
/// Bodies may not exceed this.
pub const MAX_BODY: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Request {
    pub method: String,
    /// The decoded path, without the query string.
    pub path: String,
    /// Decoded query parameters, in order.
    pub query: Vec<(String, String)>,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    /// A header's value; names are case-insensitive.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// HTTP/1.1 keeps the connection unless `Connection: close`; HTTP/1.0
    /// closes unless `Connection: keep-alive`.
    pub fn keep_alive(&self) -> bool {
        let connection = self.header("connection").map(str::to_ascii_lowercase);
        match self.version.as_str() {
            "HTTP/1.1" => connection.as_deref() != Some("close"),
            _ => connection.as_deref() == Some("keep-alive"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Response {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// A `text/plain` response.
    pub fn text(status: u16, body: &str) -> Self {
        Response::new(status)
            .with_header("Content-Type", "text/plain; charset=utf-8")
            .with_body(body.as_bytes().to_vec())
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// The bytes on the wire: status line, headers (plus `Content-Length`
    /// and `Connection`), a blank line, the body.
    pub fn to_bytes(&self, keep_alive: bool) -> Vec<u8> {
        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status, reason(self.status));
        for (name, value) in &self.headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        head.push_str(if keep_alive {
            "Connection: keep-alive\r\n"
        } else {
            "Connection: close\r\n"
        });
        head.push_str("\r\n");
        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

pub fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Content Too Large",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("malformed request: {0}")]
    Malformed(String),
    #[error("request head too large")]
    HeadTooLarge,
    #[error("body too large")]
    BodyTooLarge,
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl HttpError {
    /// The response for a request that couldn't be read (`None`: just close).
    pub fn response(&self) -> Option<Response> {
        match self {
            HttpError::Malformed(why) => Some(Response::text(400, &format!("{why}\n"))),
            HttpError::HeadTooLarge => Some(Response::text(431, "request head too large\n")),
            HttpError::BodyTooLarge => Some(Response::text(413, "body too large\n")),
            HttpError::Io(_) => None,
        }
    }
}

/// `%41` -> `A`, and `+` -> space if `plus_is_space` (query strings).
/// `None` for a broken escape or invalid UTF-8.
pub fn percent_decode(s: &str, plus_is_space: bool) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                let hex = s.get(i + 1..i + 3)?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 3;
            }
            b'+' if plus_is_space => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// Split `/a/b?x=1&y=two+words` into the decoded path and query pairs.
pub fn split_target(target: &str) -> Result<(String, Vec<(String, String)>), HttpError> {
    let bad = || HttpError::Malformed(format!("bad request target {target:?}"));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if !path.starts_with('/') {
        return Err(bad());
    }
    let path = percent_decode(path, false).ok_or_else(bad)?;
    let mut pairs = Vec::new();
    for part in query.split('&').filter(|p| !p.is_empty()) {
        let (k, v) = part.split_once('=').unwrap_or((part, ""));
        pairs.push((
            percent_decode(k, true).ok_or_else(bad)?,
            percent_decode(v, true).ok_or_else(bad)?,
        ));
    }
    Ok((path, pairs))
}

/// Parse the request line and headers (the text before the blank line).
pub fn parse_head(head: &str) -> Result<Request, HttpError> {
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let parts: Vec<&str> = request_line.split(' ').collect();
    let [method, target, version] = parts[..] else {
        return Err(HttpError::Malformed(format!(
            "bad request line {request_line:?}"
        )));
    };
    if !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
        return Err(HttpError::Malformed(format!(
            "unsupported version {version:?}"
        )));
    }
    if method.is_empty() || !method.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(HttpError::Malformed(format!("bad method {method:?}")));
    }
    let (path, query) = split_target(target)?;
    let mut headers = Vec::new();
    for line in lines.filter(|l| !l.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| HttpError::Malformed(format!("bad header {line:?}")))?;
        if name.is_empty() || name.contains(' ') {
            return Err(HttpError::Malformed(format!("bad header name {name:?}")));
        }
        headers.push((name.to_string(), value.trim().to_string()));
    }
    Ok(Request {
        method: method.to_string(),
        path,
        query,
        version: version.to_string(),
        headers,
        body: Vec::new(),
    })
}

/// Read one request. `Ok(None)` if the client closed the connection before
/// sending anything (the normal end of a keep-alive connection).
pub async fn read_request<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Option<Request>, HttpError> {
    let mut head = String::new();
    loop {
        let mut line = String::new();
        // `take` bounds how much one line may buffer.
        let n = (&mut *reader)
            .take((MAX_HEAD + 1 - head.len()) as u64)
            .read_line(&mut line)
            .await?;
        if n == 0 {
            return if head.is_empty() {
                Ok(None)
            } else {
                Err(HttpError::Malformed("connection closed mid-request".into()))
            };
        }
        head.push_str(&line);
        if head.len() > MAX_HEAD {
            return Err(HttpError::HeadTooLarge);
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if !line.ends_with('\n') {
            return Err(HttpError::Malformed("connection closed mid-request".into()));
        }
    }
    let head = head.replace("\r\n", "\n").replace('\n', "\r\n");
    let mut request = parse_head(head.trim_end())?;
    let length = match request.header("content-length") {
        Some(v) => v
            .parse::<usize>()
            .map_err(|_| HttpError::Malformed(format!("bad Content-Length {v:?}")))?,
        None => 0,
    };
    if length > MAX_BODY {
        return Err(HttpError::BodyTooLarge);
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;
    request.body = body;
    Ok(Some(request))
}

/// The `Content-Type` for a file name.
pub fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "text/javascript",
        Some("json") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

/// The file under `root` for a `/static/...` path, or `None` if the path
/// tries to leave `root` (`..`, absolute components).
pub fn static_file(root: &Path, rest: &str) -> Option<PathBuf> {
    let relative = Path::new(rest);
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    Some(root.join(relative))
}

/// The application: `GET /`, `GET /echo?msg=...`, `POST /upper`,
/// `GET /static/<file>`; 405 for a known path with the wrong method, 404
/// otherwise.
pub fn route(request: &Request, static_dir: &Path) -> Response {
    let method = request.method.as_str();
    let path = request.path.as_str();
    let allow =
        |methods: &str| Response::text(405, "method not allowed\n").with_header("Allow", methods);
    match path {
        "/" => match method {
            "GET" => Response::text(200, "Hello, HTTP!\n"),
            _ => allow("GET"),
        },
        "/echo" => match method {
            "GET" => Response::text(200, request.query_param("msg").unwrap_or("")),
            _ => allow("GET"),
        },
        "/upper" => match method {
            "POST" => match String::from_utf8(request.body.clone()) {
                Ok(text) => Response::text(200, &text.to_uppercase()),
                Err(_) => Response::text(400, "body is not UTF-8\n"),
            },
            _ => allow("POST"),
        },
        _ => match path.strip_prefix("/static/") {
            Some(rest) if method == "GET" => match static_file(static_dir, rest) {
                None => Response::text(403, "forbidden\n"),
                Some(file) => match std::fs::read(&file) {
                    Ok(bytes) => Response::new(200)
                        .with_header("Content-Type", content_type(&file))
                        .with_body(bytes),
                    Err(_) => Response::text(404, "not found\n"),
                },
            },
            Some(_) => allow("GET"),
            None => Response::text(404, "not found\n"),
        },
    }
}

/// Serve requests on one connection until the client closes, asks to
/// close, or sends something unreadable (answered with 400/413/431).
pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    static_dir: Arc<PathBuf>,
) -> io::Result<()> {
    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
    loop {
        match read_request(&mut reader).await {
            Ok(None) => break,
            Ok(Some(request)) => {
                let response = route(&request, &static_dir);
                let keep_alive = request.keep_alive();
                write.write_all(&response.to_bytes(keep_alive)).await?;
                if !keep_alive {
                    break;
                }
            }
            Err(e) => {
                if let Some(response) = e.response() {
                    write.write_all(&response.to_bytes(false)).await?;
                }
                break;
            }
        }
    }
    write.shutdown().await
}

/// Accept forever.
pub async fn serve(listener: tokio::net::TcpListener, static_dir: PathBuf) -> io::Result<()> {
    let static_dir = Arc::new(static_dir);
    loop {
        let (stream, _) = listener.accept().await?;
        let dir = static_dir.clone();
        tokio::spawn(async move {
            let _ = handle_connection(stream, dir).await;
        });
    }
}
