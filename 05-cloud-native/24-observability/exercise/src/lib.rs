//! Module 24 -- Observability. YOUR WORKSPACE.
//!
//! Logs, metrics and traces for services: what happened, how much and how
//! fast, and where the time went across services.
//!
//! | File              | Exercise |
//! |-------------------|----------|
//! | `logging.rs`      | 1, 2  structured JSON logs with `tracing`; filtering and redaction |
//! | `metrics.rs`      | 3  Prometheus metrics: RED, by route template |
//! | `tracecontext.rs` | 4  the W3C `traceparent` header |
//! | `collector.rs`    | 5  a `tracing` Layer that collects spans into traces |
//! | `services.rs`     | 5  two services propagating one trace |
//! | `slo.rs`          | 6  SLOs, error budgets, burn-rate alerts |
//! | `dashboards.rs`   | 7  Grafana dashboards and Prometheus rules as code |
//! | `sampling.rs`     | bonus: consistent head-based sampling |
//! | `capture.rs`      | capturing log output in memory (provided) |

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod capture;
pub mod collector;
pub mod dashboards;
pub mod logging;
pub mod metrics;
pub mod sampling;
pub mod services;
pub mod slo;
pub mod tracecontext;

/// The `deploy/` directory: Prometheus and Grafana configuration.
pub const DEPLOY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../deploy");
