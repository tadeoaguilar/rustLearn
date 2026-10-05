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
        let requests = Family::<RequestLabels, Counter>::default();
        let duration = Family::<RouteLabels, Histogram, fn() -> Histogram>::new_with_constructor(
            histogram as fn() -> Histogram,
        );
        let in_flight = Gauge::default();
        let mut registry = Registry::default();
        // Counters get "_total" appended in the output.
        registry.register(
            "http_requests",
            "HTTP requests by method, route and status",
            requests.clone(),
        );
        registry.register(
            "http_request_duration_seconds",
            "HTTP request latency",
            duration.clone(),
        );
        registry.register(
            "http_requests_in_flight",
            "HTTP requests being served",
            in_flight.clone(),
        );
        Metrics {
            registry: Arc::new(registry),
            requests,
            duration,
            in_flight,
        }
    }

    pub fn observe(&self, method: &str, route: &str, status: u16, elapsed: Duration) {
        self.requests
            .get_or_create(&RequestLabels {
                method: method.into(),
                route: route.into(),
                status: status.to_string(),
            })
            .inc();
        self.duration
            .get_or_create(&RouteLabels {
                method: method.into(),
                route: route.into(),
            })
            .observe(elapsed.as_secs_f64());
    }

    /// The text exposition format (OpenMetrics).
    pub fn render(&self) -> String {
        let mut out = String::new();
        prometheus_client::encoding::text::encode(&mut out, &self.registry)
            .expect("writing to a String");
        out
    }

    pub fn in_flight(&self) -> &Gauge {
        &self.in_flight
    }
}

/// Middleware: count and time every request, by route template.
pub async fn track(State(metrics): State<Metrics>, request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "unmatched".to_string(), |m| m.as_str().to_string());
    let started = Instant::now();
    metrics.in_flight.inc();
    let response = next.run(request).await;
    metrics.in_flight.dec();
    metrics.observe(
        &method,
        &route,
        response.status().as_u16(),
        started.elapsed(),
    );
    response
}

/// `GET /metrics`, for Prometheus to scrape.
pub async fn metrics_handler(State(metrics): State<Metrics>) -> impl IntoResponse {
    (
        [(
            CONTENT_TYPE,
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )],
        metrics.render(),
    )
}
