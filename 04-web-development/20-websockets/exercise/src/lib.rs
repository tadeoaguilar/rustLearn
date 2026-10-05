//! Module 20 -- 20-websockets. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m20-websockets -- <exercise number>
//!     cargo test -p m20-websockets-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod client;
pub mod protocol;
pub mod server;
pub mod state;

/// Starts a router on a background task; returns "127.0.0.1:port".
pub async fn spawn(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("local addr").to_string();
    tokio::spawn(async move { axum::serve(listener, router).await.expect("server") });
    addr
}
