use crate::sut::*;
use capture::Captured;
use collector::{FinishedSpan, SpanCollector, render_tree};
use logging::{CheckoutError, Order, Redacted, checkout, json_subscriber, mask_email, noisy};
use metrics::Metrics;
use services::{Frontend, backend, frontend, spawn};
use slo::{Alert, Counts, Slo};
use std::time::Duration;
use tracecontext::TraceParent;
use tracing_subscriber::layer::SubscriberExt;

fn order() -> Order {
    Order {
        id: "o-42".into(),
        email: "alice@example.com".into(),
        items: vec![("BOOK-1".into(), 2, 3999), ("MUG-7".into(), 1, 1250)],
    }
}

fn logs_of(directives: &str, f: impl FnOnce()) -> Vec<serde_json::Value> {
    let captured = Captured::new();
    tracing::subscriber::with_default(json_subscriber(captured.clone(), directives), f);
    captured.json_lines()
}

// ---- Exercise 1: structured logs -----------------------------------------------------------------

#[test]
fn ex1_json_events_carry_fields_and_span_context() {
    let lines = logs_of("info", || {
        assert_eq!(checkout(&order()), Ok(9248));
    });
    assert_eq!(lines.len(), 1, "{lines:#?}");
    let e = &lines[0];
    assert_eq!(e["level"], "INFO");
    assert_eq!(e["message"], "order accepted");
    assert_eq!(e["total_cents"], 9248, "fields at the top level");
    assert_eq!(e["span"]["name"], "checkout");
    assert_eq!(e["span"]["order_id"], "o-42");
    assert_eq!(e["span"]["items"], 2);
    assert!(e["timestamp"].is_string() && e["target"].as_str().unwrap().ends_with("logging"));
}

#[test]
fn ex1_rejections_are_warnings_with_a_reason() {
    let lines = logs_of("info", || {
        assert_eq!(
            checkout(&Order {
                items: vec![],
                ..order()
            }),
            Err(CheckoutError::Empty)
        );
    });
    assert_eq!(lines[0]["level"], "WARN");
    assert_eq!(lines[0]["reason"], "empty");
    assert_eq!(lines[0]["span"]["items"], 0);
}

// ---- Exercise 2: filtering and redaction -----------------------------------------------------------

#[test]
fn ex2_per_module_filters() {
    let all = logs_of("debug", noisy::chatter);
    assert_eq!(all.len(), 2);
    let quiet = logs_of(&format!("debug,{}=warn", noisy::TARGET), noisy::chatter);
    assert_eq!(quiet.len(), 1);
    assert_eq!(quiet[0]["level"], "WARN");
    assert_eq!(logs_of("error", noisy::chatter).len(), 0);
}

#[test]
fn ex2_no_personal_data_in_logs() {
    let lines = logs_of("info", || {
        let _ = checkout(&order());
    });
    let text = serde_json::to_string(&lines).unwrap();
    assert!(!text.contains("alice@example.com"), "{text}");
    assert_eq!(lines[0]["span"]["customer"], "a***@example.com");
    assert_eq!(mask_email("bob@x.io"), "b***@x.io");
    assert_eq!(mask_email("not an email"), "***");
    assert_eq!(mask_email("@x.io"), "***");
    assert_eq!(
        format!("{} {:?}", Redacted("hunter2"), Redacted(42)),
        "*** ***"
    );
}

// ---- Exercise 3: metrics -----------------------------------------------------------------------------

#[test]
fn ex3_exposition_format() {
    let m = Metrics::new();
    m.observe("GET", "/orders/{id}", 200, Duration::from_millis(30));
    m.observe("GET", "/orders/{id}", 200, Duration::from_millis(70));
    m.observe("GET", "/orders/{id}", 500, Duration::from_millis(2));
    let text = m.render();
    assert!(text.contains("# TYPE http_requests counter"), "{text}");
    assert!(
        text.contains(r#"http_requests_total{method="GET",route="/orders/{id}",status="200"} 2"#),
        "{text}"
    );
    assert!(
        text.contains(r#"http_requests_total{method="GET",route="/orders/{id}",status="500"} 1"#),
        "{text}"
    );
    assert!(
        text.contains(
            r#"http_request_duration_seconds_count{method="GET",route="/orders/{id}"} 3"#
        ),
        "{text}"
    );
    assert!(
        text.contains(
            r#"http_request_duration_seconds_bucket{le="0.05",method="GET",route="/orders/{id}"} 2"#
        ),
        "{text}"
    );
    assert!(text.contains("http_requests_in_flight"), "{text}");
    assert!(text.trim_end().ends_with("# EOF"));
}

#[tokio::test]
async fn ex3_middleware_labels_by_route_template() {
    let m = Metrics::new();
    let url = spawn(backend(m.clone())).await;
    let client = reqwest::Client::new();
    for sku in ["A", "B", "C"] {
        assert_eq!(
            client
                .get(format!("{url}/price/{sku}"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    assert_eq!(
        client
            .get(format!("{url}/missing/1"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    let r = client.get(format!("{url}/metrics")).send().await.unwrap();
    assert!(
        r.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("application/openmetrics-text")
    );
    let text = r.text().await.unwrap();
    assert!(
        text.contains(r#"route="/price/{sku}",status="200"} 3"#),
        "one series for all SKUs:\n{text}"
    );
    assert!(
        text.contains(r#"route="unmatched",status="404"} 1"#),
        "{text}"
    );
    assert!(
        !text.contains("/price/A"),
        "raw paths must never become labels"
    );
}

// ---- Exercise 4: trace context ---------------------------------------------------------------------

#[test]
fn ex4_traceparent_round_trip_and_validation() {
    let h = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
    let tp = TraceParent::parse(h).unwrap();
    assert_eq!(tp.trace_id, 0x4bf92f3577b34da6a3ce929d0e0e4736);
    assert_eq!(tp.span_id, 0x00f067aa0ba902b7);
    assert!(tp.sampled);
    assert_eq!(tp.header(), h);
    assert_eq!(
        (tp.trace_id_hex().len(), tp.span_id_hex()),
        (32, "00f067aa0ba902b7".to_string())
    );
    assert!(
        !TraceParent::parse("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00")
            .unwrap()
            .sampled
    );
    for bad in [
        "",
        "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        "00-00000000000000000000000000000000-00f067aa0ba902b7-01",
        "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01",
        "00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01",
        "00-4bf92f3577b34da6a3ce929d0e0e473-00f067aa0ba902b7-01",
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7",
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902bz-01",
    ] {
        assert_eq!(TraceParent::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn ex4_children_share_the_trace() {
    let root = TraceParent::new_root(true);
    assert!(root.trace_id != 0 && root.span_id != 0);
    let child = root.child();
    assert_eq!((child.trace_id, child.sampled), (root.trace_id, true));
    assert_ne!(child.span_id, root.span_id);
    assert_ne!(TraceParent::new_root(false).trace_id, root.trace_id);
}

// ---- Exercise 5: distributed tracing ----------------------------------------------------------------

async fn traced_call(
    collector: &SpanCollector,
    n: u32,
    traceparent: Option<&str>,
) -> Vec<FinishedSpan> {
    let backend_url = spawn(backend(Metrics::new())).await;
    let fe = Frontend {
        backend_url,
        client: reqwest::Client::new(),
    };
    let url = spawn(frontend(fe, Metrics::new())).await;
    let mut req = reqwest::Client::new().get(format!("{url}/checkout/{n}"));
    if let Some(tp) = traceparent {
        req = req.header("traceparent", tp);
    }
    let body: serde_json::Value = req.send().await.unwrap().json().await.unwrap();
    assert_eq!(body["items"], n);
    tokio::time::sleep(Duration::from_millis(50)).await;
    collector.spans()
}

// The subscriber is set for this thread; a current-thread runtime keeps
// every task (both servers, the client) on it.
#[tokio::test(flavor = "current_thread")]
async fn ex5_one_trace_across_two_services() {
    let collector = SpanCollector::new();
    let _guard =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(collector.clone()));
    let spans = traced_call(&collector, 2, None).await;

    let root = spans
        .iter()
        .find(|s| s.name == "http.server" && s.service == "frontend")
        .expect("frontend server span");
    assert_eq!(root.parent_id, None, "no incoming traceparent: a new trace");
    assert_eq!(
        root.attributes.get("http.route").map(String::as_str),
        Some("/checkout/{n}")
    );
    assert_eq!(
        root.attributes.get("http.status").map(String::as_str),
        Some("200")
    );
    let trace: Vec<&FinishedSpan> = spans
        .iter()
        .filter(|s| s.trace_id == root.trace_id)
        .collect();
    let count = |service: &str, name: &str| {
        trace
            .iter()
            .filter(|s| s.service == service && s.name == name)
            .count()
    };
    assert_eq!(
        (
            count("frontend", "http.client"),
            count("backend", "http.server"),
            count("backend", "db.query")
        ),
        (2, 2, 2)
    );

    // Every backend server span's parent is one of the frontend's client spans.
    let client_ids: Vec<&str> = trace
        .iter()
        .filter(|s| s.name == "http.client")
        .map(|s| s.span_id.as_str())
        .collect();
    for s in trace
        .iter()
        .filter(|s| s.service == "backend" && s.name == "http.server")
    {
        assert!(
            client_ids.contains(&s.parent_id.as_deref().unwrap()),
            "{s:#?}"
        );
    }
    for c in trace.iter().filter(|s| s.name == "http.client") {
        assert_eq!(c.parent_id.as_deref(), Some(root.span_id.as_str()));
    }
    assert_eq!(
        render_tree(&spans, &root.trace_id),
        "frontend http.server http.route=/checkout/{n}\n  frontend http.client\n    backend http.server http.route=/price/{sku}\n      backend db.query\n  frontend http.client\n    backend http.server http.route=/price/{sku}\n      backend db.query\n"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn ex5_continues_an_incoming_trace() {
    let collector = SpanCollector::new();
    let _guard =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(collector.clone()));
    let spans = traced_call(
        &collector,
        1,
        Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"),
    )
    .await;
    let root = spans
        .iter()
        .find(|s| s.name == "http.server" && s.service == "frontend")
        .unwrap();
    assert_eq!(root.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
    assert_eq!(root.parent_id.as_deref(), Some("00f067aa0ba902b7"));
    assert!(
        spans.iter().all(|s| s.trace_id == root.trace_id),
        "every span joins the caller's trace"
    );
    assert!(
        spans.iter().all(|s| s.span_id.len() == 16),
        "span ids are 16 hex digits"
    );
}

#[test]
fn ex5_spans_outside_a_trace_are_not_collected() {
    let collector = SpanCollector::new();
    tracing::subscriber::with_default(
        tracing_subscriber::registry().with(collector.clone()),
        || {
            let _s = tracing::info_span!("background.job").entered();
        },
    );
    assert!(collector.spans().is_empty());
}

// ---- Exercise 6: SLOs ---------------------------------------------------------------------------------

#[test]
fn ex6_budget_and_burn_rate() {
    let slo = Slo { objective: 0.999 };
    assert!((slo.error_budget() - 0.001).abs() < 1e-12);
    assert_eq!(Slo::availability(Counts::new(0, 0)), 1.0);
    let c = Counts::new(9_993_000, 10_000_000);
    assert!((Slo::availability(c) - 0.9993).abs() < 1e-9);
    assert!((slo.burn_rate(c) - 0.7).abs() < 1e-6);
    assert!((slo.budget_remaining(c) - 0.3).abs() < 1e-6);
    assert!(
        slo.budget_remaining(Counts::new(980, 1000)) < 0.0,
        "overspent"
    );
}

#[test]
fn ex6_multiwindow_alerts() {
    let slo = Slo { objective: 0.999 };
    let fine = Counts::new(100_000, 100_000);
    let on_fire = Counts::new(980, 1000); // 2% errors = burn rate 20
    let simmering = Counts::new(9_920, 10_000); // 0.8% = burn rate 8
    assert_eq!(slo.evaluate(on_fire, on_fire, on_fire, fine), Alert::Page);
    assert_eq!(
        slo.evaluate(fine, on_fire, on_fire, fine),
        Alert::None,
        "already recovered: the short window is clean"
    );
    assert_eq!(
        slo.evaluate(simmering, simmering, simmering, simmering),
        Alert::Ticket
    );
    assert_eq!(
        slo.evaluate(on_fire, fine, fine, fine),
        Alert::None,
        "a 5-minute blip alone pages nobody"
    );
    assert_eq!(slo.evaluate(fine, fine, fine, fine), Alert::None);
}

// ---- Exercise 7: dashboards and rules as code -----------------------------------------------------------

#[test]
fn ex7_red_dashboard() {
    let d = dashboards::red_dashboard("backend");
    assert_eq!(d["uid"], "red-backend");
    let panels = d["panels"].as_array().unwrap();
    assert_eq!(panels.len(), 3);
    let exprs: Vec<&str> = panels
        .iter()
        .map(|p| p["targets"][0]["expr"].as_str().unwrap())
        .collect();
    assert!(exprs[0].contains("rate(http_requests_total{job=\"backend\"}[5m])"));
    assert!(exprs[1].contains("status=~\"5..\""));
    assert!(
        exprs[2].starts_with("histogram_quantile(0.99")
            && exprs[2].contains("http_request_duration_seconds_bucket")
    );
    assert!(
        panels
            .iter()
            .all(|p| p["type"] == "timeseries" && p["datasource"]["type"] == "prometheus")
    );
}

#[test]
fn ex7_burn_rate_rules() {
    let rules = dashboards::burn_rate_rules("frontend", 0.999);
    let list = rules["groups"][0]["rules"].as_array().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["labels"]["severity"], "page");
    let fast = list[0]["expr"].as_str().unwrap();
    assert!(
        fast.contains("[1h]") && fast.contains("[5m]") && fast.contains("14.4"),
        "{fast}"
    );
    let slow = list[1]["expr"].as_str().unwrap();
    assert!(slow.contains("[6h]") && slow.contains("[30m]"), "{slow}");
    #[cfg(not(feature = "mine"))]
    {
        // The committed files must match the generator: cargo run -p m24-observability-solution -- generate
        let committed: serde_json::Value = serde_yaml_ng::from_str(
            &std::fs::read_to_string(format!("{}/prometheus/rules.yaml", crate::sut::DEPLOY_DIR))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(committed, rules);
        let dash: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(format!(
                "{}/grafana/dashboards/red-backend.json",
                crate::sut::DEPLOY_DIR
            ))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(dash, dashboards::red_dashboard("backend"));
    }
}

// ---- Bonus: sampling ------------------------------------------------------------------------------------

#[test]
fn bonus_consistent_sampling() {
    let parent = TraceParent::new_root(false);
    assert!(
        !sampling::should_sample(Some(&parent), 1, 1.0),
        "follow the parent"
    );
    assert!(sampling::should_sample(
        Some(&TraceParent {
            sampled: true,
            ..parent
        }),
        1,
        0.0
    ));
    assert!(sampling::should_sample(None, 12345, 1.0));
    assert!(!sampling::should_sample(None, 12345, 0.0));
    // Same trace id, same answer -- in every service.
    let id = TraceParent::new_root(true).trace_id;
    assert_eq!(
        sampling::should_sample(None, id, 0.3),
        sampling::should_sample(None, id, 0.3)
    );
    let kept = (0..10_000)
        .filter(|_| sampling::should_sample(None, TraceParent::new_root(true).trace_id, 0.25))
        .count();
    assert!((2200..2800).contains(&kept), "about 25%: {kept}");
}
