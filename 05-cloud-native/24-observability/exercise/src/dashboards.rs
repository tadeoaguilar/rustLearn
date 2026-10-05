//! Exercise 7: dashboards and alerts as code.
//!
//! Dashboards clicked together in a UI drift, can't be reviewed and get lost.
//! Generated from code they're versioned, reviewed and identical in every
//! environment. This generates a Grafana dashboard with the RED panels and a
//! Prometheus rules file with the burn-rate alerts from Exercise 6, for the
//! metrics of Exercise 3.

use serde_json::{Value, json};

/// The PromQL for a service's Rate, Errors and Duration. `job` is the label
/// Prometheus adds from the scrape configuration.
pub fn red_queries(job: &str) -> [(&'static str, String); 3] {
    todo!("Exercise 7")
}

/// A Grafana dashboard (the JSON model) with one time-series panel per query.
pub fn red_dashboard(job: &str) -> Value {
    todo!("Exercise 7")
}

/// Prometheus alerting rules for an SLO: the page and ticket alerts of
/// `Slo::evaluate`, as PromQL.
pub fn burn_rate_rules(job: &str, objective: f64) -> Value {
    todo!("Exercise 7")
}
