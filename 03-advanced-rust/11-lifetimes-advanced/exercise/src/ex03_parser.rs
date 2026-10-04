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
        todo!("Exercise 3")
    }
}

impl std::error::Error for ParseError<'_> {}

pub fn parse_request<'a>(raw: &'a str) -> Result<Request<'a>, ParseError<'a>> {
    todo!("Exercise 3")
}

impl<'a> Request<'a> {
    /// `&'a str`: the value outlives this Request, as long as the raw input lives.
    /// `name` gets its own (elided) lifetime -- it's only compared, never returned.
    // TODO: should the value borrow from the Request, or from the raw input?
    pub fn header(&self, name: &str) -> Option<&str> {
        todo!("Exercise 3")
    }

    pub fn headers(&self) -> &[(&'a str, &'a str)] {
        todo!("Exercise 3")
    }

    /// "/search?q=rust&page=2" -> [("q", "rust"), ("page", "2")].
    /// A key without '=' gets an empty value. No percent-decoding: that would
    /// need to allocate, which a zero-copy API can't do.
    pub fn query_params(&self) -> Vec<(&'a str, &'a str)> {
        todo!("Exercise 3")
    }
}

pub const SAMPLE: &str = "POST /search?q=rust&page=2 HTTP/1.1\r\nHost: example.com\r\nContent-Type: text/plain\r\n\r\nhello body";

pub fn run() {
    todo!("Exercise 3")
}
