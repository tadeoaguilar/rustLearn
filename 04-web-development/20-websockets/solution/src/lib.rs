//! Module 20 -- WebSockets. Reference solution.
//!
//! HTTP is request/response: the client asks, the server answers, done. A
//! WebSocket starts as an HTTP request (`Upgrade: websocket`) and then stays
//! open as a two-way message channel -- the server can push whenever it
//! likes. That's what chat, notifications, live dashboards and collaborative
//! editors need.
//!
//! | File          | Exercise |
//! |---------------|----------|
//! | `protocol.rs` | 2  typed JSON messages, both directions |
//! | `server.rs`   | 1, 3-7  the endpoints |
//! | `state.rs`    | 3, 4, 6  rooms, subscriptions, the shared document |
//! | `client.rs`   | 1-7 a small client for tests and demos; bonus: reconnection with backoff |

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
