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
    let incoming = request
        .headers()
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .and_then(TraceParent::parse);
    // Our span: the caller's trace with a new span id -- or a new trace.
    let current = incoming.map_or_else(|| TraceParent::new_root(true), |p| p.child());
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or("unmatched", |m| m.as_str())
        .to_string();
    let span = info_span!(
        "http.server",
        service,
        trace_id = %current.trace_id_hex(),
        span_id = %current.span_id_hex(),
        parent_id = %incoming.map(|p| p.span_id_hex()).unwrap_or_default(),
        http.method = %request.method(),
        http.route = %route,
        http.status = field::Empty,
    );
    request.extensions_mut().insert(current);
    let response = next.run(request).instrument(span.clone()).await;
    span.record("http.status", response.status().as_u16());
    response
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
    let mut total = 0;
    for i in 0..n {
        // A client span per call; its id goes out as the parent in traceparent.
        let outgoing = current.child();
        let sku = format!("SKU-{i}");
        let url = format!("{}/price/{sku}", fe.backend_url);
        let span = info_span!("http.client", span_id = %outgoing.span_id_hex(), http.url = %url, order = i);
        let price: u64 = async {
            let r = fe
                .client
                .get(&url)
                .header("traceparent", outgoing.header())
                .send()
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)?;
            r.json::<u64>().await.map_err(|_| StatusCode::BAD_GATEWAY)
        }
        .instrument(span)
        .await?;
        total += price;
    }
    info!(total, "checkout priced");
    Ok(Json(
        serde_json::json!({ "items": n, "total_cents": total }),
    ))
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
