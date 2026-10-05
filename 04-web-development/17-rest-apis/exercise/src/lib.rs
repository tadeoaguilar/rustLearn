//! Module 17 -- 17-rest-apis. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m17-rest-apis -- <exercise number>
//!     cargo test -p m17-rest-apis-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod api;
pub mod client;
pub mod model;
pub mod openapi;
pub mod pagination;
pub mod problem;
pub mod rate_limit;
pub mod validation;

/// Starts `router` on a background task; returns its base URL.
pub async fn spawn(router: axum::Router, addr: &str) -> String {
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let local = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, router).await.expect("server") });
    format!("http://{local}")
}
