//! Exercise 5: two services, one trace.
//!
//! `frontend` serves `GET /checkout/{n}` by calling `backend`'s
//! `GET /price/{sku}` n times. Each service's middleware continues the trace
//! from the incoming `traceparent` header (or starts one), and the frontend
//! sends a `traceparent` on its outgoing calls. The result, in any tracing
//! backend: one trace, showing where the time went across both services.

use crate::metrics::{Metrics, metrics_handler, track};
use crate::tracecontext::TraceParent;
use axum::extract::{MatchedPath, Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use axum::{Extension, Json, Router};
use std::time::Duration;
use tracing::{Instrument, field, info, info_span};

/// Middleware: one `http.server` span per request, continuing the caller's
/// trace. Handlers get the current span's ids as `Extension<TraceParent>`.
pub async fn trace_requests(
    State(service): State<&'static str>,
    mut request: Request,
    next: Next,
) -> Response {
    todo!("Exercise 5")
}

#[derive(Clone)]
pub struct Frontend {
    pub backend_url: String,
    pub client: reqwest::Client,
}

async fn checkout(
    State(fe): State<Frontend>,
    Extension(current): Extension<TraceParent>,
    Path(n): Path<u32>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    todo!("Exercise 5")
}

async fn price(Path(sku): Path<String>) -> Json<u64> {
    let cents = async {
        tokio::time::sleep(Duration::from_millis(3)).await; // "the database"
        1000 + sku.len() as u64
    }
    .instrument(info_span!(
        "db.query",
        db.statement = "SELECT price FROM items WHERE sku = $1"
    ))
    .await;
    Json(cents)
}

pub fn frontend(fe: Frontend, metrics: Metrics) -> Router {
    Router::new()
        .route("/checkout/{n}", get(checkout))
        .with_state(fe)
        .layer(middleware::from_fn_with_state("frontend", trace_requests))
        .layer(middleware::from_fn_with_state(metrics.clone(), track))
        .route("/metrics", get(metrics_handler).with_state(metrics))
}

pub fn backend(metrics: Metrics) -> Router {
    Router::new()
        .route("/price/{sku}", get(price))
        .layer(middleware::from_fn_with_state("backend", trace_requests))
        .layer(middleware::from_fn_with_state(metrics.clone(), track))
        .route("/metrics", get(metrics_handler).with_state(metrics))
}

/// Starts a router on 127.0.0.1:<random port>; returns "http://127.0.0.1:port".
pub async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.expect("server") });
    url
}
