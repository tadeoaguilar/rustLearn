//! Module 19 -- 19-authentication. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m19-authentication -- <exercise number>
//!     cargo test -p m19-authentication-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod api_keys;
pub mod app;
pub mod mock_provider;
pub mod oauth;
pub mod password;
pub mod rbac;
pub mod sessions;
pub mod tokens;
pub mod totp;
pub mod users;

/// Random bytes from the OS-seeded generator, hex encoded.
pub fn random_hex(bytes: usize) -> String {
    (0..bytes)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect()
}

/// Starts a router on a background task; returns its base URL.
pub async fn spawn(router: axum::Router, addr: &str) -> String {
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    let local = listener.local_addr().expect("local addr");
    tokio::spawn(async move { axum::serve(listener, router).await.expect("server") });
    format!("http://{local}")
}
