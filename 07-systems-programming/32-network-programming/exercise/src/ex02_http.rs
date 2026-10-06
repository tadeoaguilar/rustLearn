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
        todo!("Exercise 2")
    }

    pub fn query_param(&self, name: &str) -> Option<&str> {
        todo!("Exercise 2")
    }

    /// HTTP/1.1 keeps the connection unless `Connection: close`; HTTP/1.0
    /// closes unless `Connection: keep-alive`.
    pub fn keep_alive(&self) -> bool {
        todo!("Exercise 2")
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
        todo!("Exercise 2")
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
        todo!("Exercise 2")
    }
}

/// `%41` -> `A`, and `+` -> space if `plus_is_space` (query strings).
/// `None` for a broken escape or invalid UTF-8.
pub fn percent_decode(s: &str, plus_is_space: bool) -> Option<String> {
    todo!("Exercise 2")
}

/// Split `/a/b?x=1&y=two+words` into the decoded path and query pairs.
pub fn split_target(target: &str) -> Result<(String, Vec<(String, String)>), HttpError> {
    todo!("Exercise 2")
}

/// Parse the request line and headers (the text before the blank line).
pub fn parse_head(head: &str) -> Result<Request, HttpError> {
    todo!("Exercise 2")
}

/// Read one request. `Ok(None)` if the client closed the connection before
/// sending anything (the normal end of a keep-alive connection).
pub async fn read_request<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Option<Request>, HttpError> {
    todo!("Exercise 2")
}

/// The `Content-Type` for a file name.
pub fn content_type(path: &Path) -> &'static str {
    todo!("Exercise 2")
}

/// The file under `root` for a `/static/...` path, or `None` if the path
/// tries to leave `root` (`..`, absolute components).
pub fn static_file(root: &Path, rest: &str) -> Option<PathBuf> {
    todo!("Exercise 2")
}

/// The application: `GET /`, `GET /echo?msg=...`, `POST /upper`,
/// `GET /static/<file>`; 405 for a known path with the wrong method, 404
/// otherwise.
pub fn route(request: &Request, static_dir: &Path) -> Response {
    todo!("Exercise 2")
}

/// Serve requests on one connection until the client closes, asks to
/// close, or sends something unreadable (answered with 400/413/431).
pub async fn handle_connection(
    stream: tokio::net::TcpStream,
    static_dir: Arc<PathBuf>,
) -> io::Result<()> {
    todo!("Exercise 2")
}

/// Accept forever.
pub async fn serve(listener: tokio::net::TcpListener, static_dir: PathBuf) -> io::Result<()> {
    todo!("Exercise 2")
}
