//! Upstream services for tests and the demo. PROVIDED.

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum Behaviour {
    Healthy,
    /// The first N requests get 503.
    FailFirst(u64),
    /// Every request gets this status.
    AlwaysFail(u16),
    /// Every request takes this long.
    Slow(Duration),
}

#[derive(Clone)]
struct Upstream {
    name: String,
    behaviour: Behaviour,
    hits: Arc<AtomicU64>,
}

/// Echoes what it received, as JSON: who it is, the path, and the headers a
/// proxy is expected to set or strip.
async fn handle(State(u): State<Upstream>, req: Request) -> Response {
    let n = u.hits.fetch_add(1, Ordering::SeqCst) + 1;
    match u.behaviour {
        Behaviour::FailFirst(k) if n <= k => {
            return (StatusCode::SERVICE_UNAVAILABLE, "warming up").into_response();
        }
        Behaviour::AlwaysFail(code) => {
            return (StatusCode::from_u16(code).unwrap(), "broken").into_response();
        }
        Behaviour::Slow(d) => tokio::time::sleep(d).await,
        _ => {}
    }
    let header = |name: &str| {
        req.headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    Json(serde_json::json!({
        "upstream": u.name,
        "method": req.method().as_str(),
        "path": req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/"),
        "x-forwarded-for": header("x-forwarded-for"),
        "x-request-id": header("x-request-id"),
        "x-secret-hop": header("x-secret-hop"),
        "keep-alive": header("keep-alive"),
    }))
    .into_response()
}

/// Starts an upstream on a random port: ("http://127.0.0.1:port", hit counter).
pub async fn spawn_upstream(name: &str, behaviour: Behaviour) -> (String, Arc<AtomicU64>) {
    let hits = Arc::new(AtomicU64::new(0));
    let state = Upstream {
        name: name.into(),
        behaviour,
        hits: hits.clone(),
    };
    let router = Router::new().fallback(handle).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.expect("upstream") });
    (url, hits)
}
