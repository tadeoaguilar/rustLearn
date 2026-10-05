//! Exercise 3: Prometheus metrics.
//!
//! Logs tell you what happened to *one* request; metrics tell you what's
//! happening to *all* of them, cheaply: a few numbers per time series,
//! scraped every 15 s. The "RED" method covers a request-driven service:
//! **R**ate, **E**rrors, **D**uration.
//!
//! Labels multiply series: one series per distinct label combination. So
//! label by the route *template* (`/orders/{id}`), never the raw path
//! (`/orders/1`, `/orders/2`, ... -- unbounded cardinality, a dead Prometheus).

use axum::extract::{MatchedPath, Request, State};
use axum::http::header::CONTENT_TYPE;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RequestLabels {
    pub method: String,
    pub route: String,
    pub status: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct RouteLabels {
    pub method: String,
    pub route: String,
}

/// Buckets from 5 ms to ~10 s. Pick them around your latency objective.
pub const BUCKETS: [f64; 12] = [
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
];

fn histogram() -> Histogram {
    Histogram::new(BUCKETS)
}

#[derive(Clone)]
pub struct Metrics {
    registry: Arc<Registry>,
    requests: Family<RequestLabels, Counter>,
    duration: Family<RouteLabels, Histogram, fn() -> Histogram>,
    in_flight: Gauge,
}

impl Default for Metrics {
    fn default() -> Metrics {
        Metrics::new()
    }
}

impl Metrics {
    pub fn new() -> Metrics {
        todo!("Exercise 3")
    }

    pub fn observe(&self, method: &str, route: &str, status: u16, elapsed: Duration) {
        todo!("Exercise 3")
    }

    /// The text exposition format (OpenMetrics).
    pub fn render(&self) -> String {
        todo!("Exercise 3")
    }

    pub fn in_flight(&self) -> &Gauge {
        todo!("Exercise 3")
    }
}

/// Middleware: count and time every request, by route template.
pub async fn track(State(metrics): State<Metrics>, request: Request, next: Next) -> Response {
    todo!("Exercise 3")
}

/// `GET /metrics`, for Prometheus to scrape.
#[allow(unreachable_code)]
pub async fn metrics_handler(State(metrics): State<Metrics>) -> impl IntoResponse {
    todo!("Exercise 3");
    "" // placeholder so the signature compiles: replace it
}
