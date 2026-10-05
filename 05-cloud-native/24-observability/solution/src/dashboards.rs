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
    [
        (
            "Requests per second",
            format!("sum by (route) (rate(http_requests_total{{job=\"{job}\"}}[5m]))"),
        ),
        (
            "Error ratio",
            format!(
                "sum(rate(http_requests_total{{job=\"{job}\",status=~\"5..\"}}[5m])) / sum(rate(http_requests_total{{job=\"{job}\"}}[5m]))"
            ),
        ),
        (
            "p99 latency",
            format!(
                "histogram_quantile(0.99, sum by (le, route) (rate(http_request_duration_seconds_bucket{{job=\"{job}\"}}[5m])))"
            ),
        ),
    ]
}

/// A Grafana dashboard (the JSON model) with one time-series panel per query.
pub fn red_dashboard(job: &str) -> Value {
    let units = ["reqps", "percentunit", "s"];
    let panels: Vec<Value> = red_queries(job)
        .into_iter()
        .zip(units)
        .enumerate()
        .map(|(i, ((title, expr), unit))| {
            json!({
                "id": i + 1,
                "type": "timeseries",
                "title": title,
                "gridPos": { "x": (i % 2) * 12, "y": (i / 2) * 8, "w": 12, "h": 8 },
                "datasource": { "type": "prometheus", "uid": "prometheus" },
                "fieldConfig": { "defaults": { "unit": unit } },
                "targets": [{ "refId": "A", "expr": expr }],
            })
        })
        .collect();
    json!({
        "uid": format!("red-{job}"),
        "title": format!("{job} -- RED"),
        "schemaVersion": 39,
        "time": { "from": "now-1h", "to": "now" },
        "refresh": "10s",
        "panels": panels,
    })
}

/// Prometheus alerting rules for an SLO: the page and ticket alerts of
/// `Slo::evaluate`, as PromQL.
pub fn burn_rate_rules(job: &str, objective: f64) -> Value {
    let budget = 1.0 - objective;
    let ratio = |window: &str| {
        format!(
            "(sum(rate(http_requests_total{{job=\"{job}\",status=~\"5..\"}}[{window}])) / sum(rate(http_requests_total{{job=\"{job}\"}}[{window}])))"
        )
    };
    let rule = |name: &str, long: &str, short: &str, burn: f64, severity: &str| {
        json!({
            "alert": name,
            "expr": format!("{} > ({burn} * {budget:.6}) and {} > ({burn} * {budget:.6})", ratio(long), ratio(short)),
            "labels": { "severity": severity },
            "annotations": { "summary": format!("{job} is burning its error budget {burn}x too fast ({long}/{short} windows)") },
        })
    };
    json!({
        "groups": [{
            "name": format!("{job}-slo"),
            "rules": [
                rule("ErrorBudgetBurnFast", "1h", "5m", 14.4, "page"),
                rule("ErrorBudgetBurnSlow", "6h", "30m", 6.0, "ticket"),
            ],
        }]
    })
}
