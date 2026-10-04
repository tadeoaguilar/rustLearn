//! A tiny JSON API for Exercise 4 to call, so nothing depends on the network.
//! It mimics jsonplaceholder.typicode.com's `/posts/{id}`, plus two endpoints
//! for practising failure handling.
//!
//! This is infrastructure, not an exercise: the exercise crate has an
//! identical copy.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[derive(Clone, Default)]
struct MockState {
    flaky_calls: Arc<AtomicU32>,
}

/// GET /posts/{id}   -> a post, or 404 for id 0 or > 100
/// GET /flaky        -> 503 for the first two calls, then 200
/// GET /slow         -> answers after 2 seconds
pub fn router() -> Router {
    Router::new()
        .route("/posts/{id}", get(post))
        .route("/flaky", get(flaky))
        .route("/slow", get(slow))
        .with_state(MockState::default())
}

async fn post(Path(id): Path<u32>) -> Result<Json<Value>, StatusCode> {
    if id == 0 || id > 100 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(json!({
        "userId": (id - 1) / 10 + 1,
        "id": id,
        "title": format!("post number {id}"),
        "body": "lorem ipsum",
    })))
}

async fn flaky(State(state): State<MockState>) -> Result<&'static str, StatusCode> {
    let n = state.flaky_calls.fetch_add(1, Ordering::SeqCst);
    if n < 2 {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    } else {
        Ok("finally")
    }
}

async fn slow() -> &'static str {
    tokio::time::sleep(Duration::from_secs(2)).await;
    "too late"
}

/// Starts the API on 127.0.0.1 with an OS-assigned port and returns its base
/// URL, e.g. "http://127.0.0.1:53412". The server runs until the runtime shuts down.
pub async fn spawn() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind a local port");
    let addr: SocketAddr = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, router()).await.expect("mock server");
    });
    format!("http://{addr}")
}
