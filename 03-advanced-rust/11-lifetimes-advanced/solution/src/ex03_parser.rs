//! Exercise 3: A Zero-Copy Parser.
//!
//! Every `&'a str` in `Request<'a>` points into the raw input. Parsing
//! allocates only the Vec of header pairs -- no Strings. The price: a
//! `Request` can't outlive the buffer it was parsed from.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub version: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    pub body: &'a str,
}

/// Errors borrow from the input too: they can quote the offending line
/// without copying it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError<'a> {
    Empty,
    BadRequestLine(&'a str),
    BadHeader(&'a str),
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty request"),
            ParseError::BadRequestLine(l) => write!(f, "bad request line: {l:?}"),
            ParseError::BadHeader(l) => write!(f, "bad header: {l:?}"),
        }
    }
}

impl std::error::Error for ParseError<'_> {}

pub fn parse_request<'a>(raw: &'a str) -> Result<Request<'a>, ParseError<'a>> {
    // Split head and body at the first blank line (CRLF or bare LF).
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .or_else(|| raw.split_once("\n\n"))
        .unwrap_or((raw, ""));

    let mut lines = head.lines(); // lines() strips "\n" and a trailing "\r"
    let request_line = lines
        .next()
        .filter(|l| !l.trim().is_empty())
        .ok_or(ParseError::Empty)?;

    let mut parts = request_line.split_whitespace();
    let (Some(method), Some(path), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ParseError::BadRequestLine(request_line));
    };
    if !version.starts_with("HTTP/") {
        return Err(ParseError::BadRequestLine(request_line));
    }

    let headers = lines
        .filter(|l| !l.is_empty())
        .map(|line| {
            let (name, value) = line.split_once(':').ok_or(ParseError::BadHeader(line))?;
            if name.is_empty() || name.contains(char::is_whitespace) {
                return Err(ParseError::BadHeader(line));
            }
            Ok((name, value.trim()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Request {
        method,
        path,
        version,
        headers,
        body,
    })
}

impl<'a> Request<'a> {
    /// `&'a str`: the value outlives this Request, as long as the raw input lives.
    /// `name` gets its own (elided) lifetime -- it's only compared, never returned.
    pub fn header(&self, name: &str) -> Option<&'a str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|&(_, v)| v)
    }

    pub fn headers(&self) -> &[(&'a str, &'a str)] {
        &self.headers
    }

    /// "/search?q=rust&page=2" -> [("q", "rust"), ("page", "2")].
    /// A key without '=' gets an empty value. No percent-decoding: that would
    /// need to allocate, which a zero-copy API can't do.
    pub fn query_params(&self) -> Vec<(&'a str, &'a str)> {
        let Some((_, query)) = self.path.split_once('?') else {
            return Vec::new();
        };
        query
            .split('&')
            .filter(|p| !p.is_empty())
            .map(|pair| pair.split_once('=').unwrap_or((pair, "")))
            .collect()
    }
}

pub const SAMPLE: &str = "POST /search?q=rust&page=2 HTTP/1.1\r\nHost: example.com\r\nContent-Type: text/plain\r\n\r\nhello body";

pub fn run() {
    let raw = String::from(SAMPLE);
    let req = parse_request(&raw).expect("sample parses");
    println!("{} {} {}", req.method, req.path, req.version);
    println!(
        "host = {:?}, CONTENT-TYPE = {:?}",
        req.header("host"),
        req.header("CONTENT-TYPE")
    );
    println!("query = {:?}, body = {:?}", req.query_params(), req.body);
    println!(
        "zero-copy: method points into raw? {}",
        std::ptr::eq(req.method.as_ptr(), raw.as_ptr())
    );
    for bad in ["", "GET\r\n\r\n", "GET / HTTP/1.1\r\nno colon here\r\n\r\n"] {
        println!("{bad:?} -> {}", parse_request(bad).unwrap_err());
    }
}
