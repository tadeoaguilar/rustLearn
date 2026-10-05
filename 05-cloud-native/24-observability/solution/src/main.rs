// Reference solution for 24-observability.
//
//     cargo run -p m24-observability-solution -- demo       # logs, metrics, a trace tree, SLO maths
//     cargo run -p m24-observability-solution -- serve      # frontend :3000, backend :3001, JSON logs on stdout
//     cargo run -p m24-observability-solution -- generate   # rewrite deploy/ dashboards and alert rules

use m24_observability_solution::capture::Captured;
use m24_observability_solution::collector::{SpanCollector, render_tree};
use m24_observability_solution::logging::{Order, checkout, json_subscriber, noisy};
use m24_observability_solution::metrics::Metrics;
use m24_observability_solution::services::{Frontend, backend, frontend, spawn};
use m24_observability_solution::slo::{Counts, Slo};
use m24_observability_solution::{DEPLOY_DIR, dashboards};
use tracing_subscriber::layer::SubscriberExt;

async fn demo() {
    // 1-2. Structured logs.
    let logs = Captured::new();
    let filter = format!("info,{}=warn", noisy::TARGET);
    tracing::subscriber::with_default(json_subscriber(logs.clone(), &filter), || {
        let order = Order {
            id: "o-42".into(),
            email: "alice@example.com".into(),
            items: vec![("BOOK-1".into(), 2, 3999)],
        };
        let _ = checkout(&order);
        let _ = checkout(&Order {
            items: vec![],
            ..order
        });
        noisy::chatter();
    });
    println!("JSON logs:\n{}", logs.text());

    // 3-5. Two services, metrics and one trace.
    let collector = SpanCollector::new();
    let subscriber = tracing_subscriber::registry().with(collector.clone());
    let _guard = tracing::subscriber::set_default(subscriber); // this thread: run the demo on a current-thread runtime
    let (fe_metrics, be_metrics) = (Metrics::new(), Metrics::new());
    let backend_url = spawn(backend(be_metrics.clone())).await;
    let fe = Frontend {
        backend_url,
        client: reqwest::Client::new(),
    };
    let frontend_url = spawn(frontend(fe, fe_metrics.clone())).await;
    let client = reqwest::Client::new();
    let body = client
        .get(format!("{frontend_url}/checkout/3"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("GET /checkout/3 -> {body}");
    let incoming = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
    client
        .get(format!("{frontend_url}/checkout/1"))
        .header("traceparent", incoming)
        .send()
        .await
        .unwrap();
    client
        .get(format!("{frontend_url}/nope"))
        .send()
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await; // let spans close

    let spans = collector.spans();
    let first = spans
        .iter()
        .find(|s| s.name == "http.server" && s.service == "frontend")
        .unwrap()
        .trace_id
        .clone();
    println!("\ntrace {first}:\n{}", render_tree(&spans, &first));
    println!(
        "trace continued from an incoming traceparent:\n{}",
        render_tree(&spans, "4bf92f3577b34da6a3ce929d0e0e4736")
    );
    let backend_metrics = be_metrics.render();
    println!("backend /metrics (excerpt):");
    for line in backend_metrics
        .lines()
        .filter(|l| l.starts_with("http_requests_total") || l.contains("_count"))
    {
        println!("  {line}");
    }
    let fe_text = fe_metrics.render();
    println!(
        "  (frontend has a series for route=\"unmatched\": {})",
        fe_text.contains("route=\"unmatched\"")
    );

    // 6. SLO maths.
    let slo = Slo { objective: 0.999 };
    let month = Counts::new(9_993_000, 10_000_000);
    println!(
        "\n99.9% SLO, 10M requests, 7,000 errors: availability {:.4}, budget remaining {:.0}%",
        Slo::availability(month),
        slo.budget_remaining(month) * 100.0
    );
    let outage = Counts::new(98, 100);
    let quiet = Counts::new(100_000, 100_000);
    println!(
        "2% errors in the last hour and 5 minutes -> {:?}",
        slo.evaluate(outage, outage, outage, quiet)
    );
    println!(
        "2% errors 5 minutes ago, recovered since  -> {:?}",
        slo.evaluate(quiet, outage, outage, quiet)
    );

    // 7.
    println!("\nRED queries for job=\"backend\":");
    for (title, q) in dashboards::red_queries("backend") {
        println!("  {title}: {q}");
    }
}

fn generate() {
    let dir = format!("{DEPLOY_DIR}/grafana/dashboards");
    std::fs::create_dir_all(&dir).unwrap();
    for job in ["frontend", "backend"] {
        let text = serde_json::to_string_pretty(&dashboards::red_dashboard(job)).unwrap() + "\n";
        std::fs::write(format!("{dir}/red-{job}.json"), text).unwrap();
    }
    let rules = serde_yaml_ng::to_string(&dashboards::burn_rate_rules("frontend", 0.999)).unwrap();
    std::fs::write(format!("{DEPLOY_DIR}/prometheus/rules.yaml"), rules).unwrap();
    println!("wrote {DEPLOY_DIR}/grafana/dashboards/*.json and prometheus/rules.yaml");
}

async fn serve() {
    use std::io::stdout;
    tracing::subscriber::set_global_default(json_subscriber(stdout, "info")).unwrap();
    let be = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    let fe = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let frontend_router = frontend(
        Frontend {
            backend_url: "http://127.0.0.1:3001".into(),
            client: reqwest::Client::new(),
        },
        Metrics::new(),
    );
    eprintln!(
        "frontend http://127.0.0.1:3000/checkout/3  backend http://127.0.0.1:3001/price/X  (/metrics on both)"
    );
    let (a, b) = tokio::join!(
        axum::serve(be, backend(Metrics::new())),
        axum::serve(fe, frontend_router)
    );
    a.unwrap();
    b.unwrap();
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    match std::env::args().nth(1).as_deref() {
        Some("demo") | Some("all") => rt.block_on(demo()),
        Some("serve") => tokio::runtime::Runtime::new().unwrap().block_on(serve()),
        Some("generate") => generate(),
        _ => {
            println!("24-observability -- reference solution\n");
            println!(
                "  cargo run -p m24-observability-solution -- demo       logs, metrics, traces, SLOs"
            );
            println!(
                "  cargo run -p m24-observability-solution -- serve      frontend :3000 + backend :3001"
            );
            println!(
                "  cargo run -p m24-observability-solution -- generate   regenerate deploy/ dashboards and rules"
            );
        }
    }
}
