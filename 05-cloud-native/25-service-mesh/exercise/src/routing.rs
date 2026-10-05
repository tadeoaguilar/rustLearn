//! Exercise 2: L7 routing and headers.
//!
//! An HTTP-aware proxy can route by path and headers, and must handle
//! headers correctly: **hop-by-hop** headers (`Connection`, `Keep-Alive`,
//! `Transfer-Encoding`, ...) describe one TCP connection and must not be
//! forwarded; `X-Forwarded-For` tells the upstream who the real client was;
//! a request id ties the hops together.

use axum::http::{HeaderMap, HeaderName, HeaderValue};
use rand::Rng;

/// Where a route sends traffic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Cluster(String),
    /// Weighted: (cluster, weight). Exercise 4.
    Split(Vec<(String, u32)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub name: String,
    pub path_prefix: String,
    /// Only matches if this header has this value.
    pub header: Option<(String, String)>,
    pub target: Target,
}

#[derive(Debug, Clone, Default)]
pub struct RouteTable {
    pub routes: Vec<Route>,
}

impl RouteTable {
    /// The most specific match: routes with a header condition beat routes
    /// without; then the longest prefix wins. Prefixes match whole segments:
    /// "/api" matches "/api" and "/api/x", not "/apix".
    pub fn route(&self, path: &str, headers: &HeaderMap) -> Option<&Route> {
        todo!("Exercise 2")
    }
}

pub const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// Removes hop-by-hop headers, including any the `Connection` header names.
pub fn strip_hop_by_hop(headers: &mut HeaderMap) {
    todo!("Exercise 2")
}

/// The headers to send upstream: hop-by-hop removed, the client's address
/// appended to X-Forwarded-For, an X-Request-Id added if there isn't one.
pub fn upstream_headers(incoming: &HeaderMap, client_ip: &str) -> HeaderMap {
    todo!("Exercise 2")
}
