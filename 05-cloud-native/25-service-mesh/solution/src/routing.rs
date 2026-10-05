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
        self.routes
            .iter()
            .filter(|r| {
                let p = r.path_prefix.trim_end_matches('/');
                path == p || path.starts_with(&format!("{p}/")) || p.is_empty()
            })
            .filter(|r| match &r.header {
                None => true,
                Some((name, value)) => {
                    headers.get(name.as_str()).and_then(|v| v.to_str().ok()) == Some(value.as_str())
                }
            })
            .max_by_key(|r| {
                (
                    r.header.is_some(),
                    r.path_prefix.trim_end_matches('/').len(),
                )
            })
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
    let named: Vec<String> = headers
        .get_all("connection")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(',').map(|s| s.trim().to_ascii_lowercase()))
        .filter(|s| !s.is_empty())
        .collect();
    for name in HOP_BY_HOP.iter().map(|s| s.to_string()).chain(named) {
        headers.remove(name.as_str());
    }
}

/// The headers to send upstream: hop-by-hop removed, the client's address
/// appended to X-Forwarded-For, an X-Request-Id added if there isn't one.
pub fn upstream_headers(incoming: &HeaderMap, client_ip: &str) -> HeaderMap {
    let mut h = incoming.clone();
    strip_hop_by_hop(&mut h);
    h.remove("host"); // the client library sets it for the upstream
    let xff = match h.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        Some(existing) => format!("{existing}, {client_ip}"),
        None => client_ip.to_string(),
    };
    h.insert(
        HeaderName::from_static("x-forwarded-for"),
        HeaderValue::from_str(&xff).expect("valid header value"),
    );
    if !h.contains_key("x-request-id") {
        let id = format!("{:016x}", rand::thread_rng().r#gen::<u64>());
        h.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(&id).unwrap(),
        );
    }
    h
}
