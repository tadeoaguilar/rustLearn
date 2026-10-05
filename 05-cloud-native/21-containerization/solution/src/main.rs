// Reference solution for 21-containerization.
//
//     cargo run -p m21-containerization-solution -- demo         # everything, locally
//     cargo run -p m21-containerization-solution -- serve        # what the container runs
//     cargo run -p m21-containerization-solution -- healthcheck  # what HEALTHCHECK runs
//     cargo run -p m21-containerization-solution -- lint <Dockerfile>

use m21_containerization_solution::config::Config;
use m21_containerization_solution::dep_watch::{WatchConfig, host_port, watch_tcp};
use m21_containerization_solution::server::{AppState, serve, shutdown_signal};
use m21_containerization_solution::{DOCKER_DIR, build_info, compose, dockerfile_lint, probe};
use std::net::SocketAddr;
use std::time::Duration;

async fn run_server() -> i32 {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return 2; // a config error is fatal: crash loudly, don't limp along
        }
    };
    let listener = match tokio::net::TcpListener::bind(config.bind_addr()).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("can't bind {}: {e}", config.bind_addr());
            return 1;
        }
    };
    let info = build_info::current();
    eprintln!(
        "{} {} ({}, {}, {} linking) listening on {}  config: {config:?}",
        info.name,
        info.version,
        info.git_sha,
        info.target,
        info.linkage,
        config.bind_addr()
    );
    let state = AppState::new(config.clone());
    if let Some(addr) = config.database_url.as_deref().and_then(host_port) {
        let watch = WatchConfig {
            interval: Duration::from_secs(2),
            timeout: Duration::from_secs(1),
            failures: 3,
            successes: 1,
        };
        tokio::spawn(watch_tcp(
            "database".into(),
            addr,
            state.health.clone(),
            watch,
        ));
    }
    let signal = async {
        shutdown_signal().await;
        eprintln!("shutdown requested: draining");
    };
    match serve(listener, state, signal).await {
        Ok(report) => {
            eprintln!(
                "stopped after {:?}, drained cleanly: {}",
                report.elapsed, report.drained_cleanly
            );
            0
        }
        Err(e) => {
            eprintln!("server error: {e}");
            1
        }
    }
}

async fn healthcheck(path: &str) -> i32 {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let result = probe::probe(
        SocketAddr::from(([127, 0, 0, 1], port)),
        path,
        Duration::from_secs(2),
    )
    .await;
    if let Err(e) = &result {
        eprintln!("unhealthy: {e:?}");
    }
    probe::exit_code(&result)
}

async fn demo() {
    // 1. Configuration: every error at once, secrets redacted.
    let bad = Config::from_lookup(|k| match k {
        "PORT" => Some("eighty".into()),
        "LOG_LEVEL" => Some("loud".into()),
        _ => None,
    });
    print!("{}", bad.unwrap_err());
    let good = Config::from_lookup(|k| match k {
        "API_TOKEN" => Some("s3cr3t".into()),
        "DRAIN_DELAY_MS" => Some("200".into()),
        _ => None,
    })
    .unwrap();
    println!("config: {good:?}\n");

    // 2-4. Serve, probe, shut down gracefully while a request is in flight.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let state = AppState::new(good);
    let health = state.health.clone();
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, state, async {
        let _ = stop_rx.await;
    }));
    tokio::time::sleep(Duration::from_millis(50)).await;
    let wait = Duration::from_secs(1);
    println!(
        "probe /healthz -> {:?}",
        probe::probe(addr, "/healthz", wait).await
    );
    health.set_dependency("database", false);
    println!(
        "probe /readyz with the database down -> {:?}",
        probe::probe(addr, "/readyz", wait).await
    );
    health.set_dependency("database", true);
    let in_flight = tokio::spawn(probe::probe(addr, "/slow?ms=400", Duration::from_secs(5)));
    tokio::time::sleep(Duration::from_millis(50)).await;
    stop_tx.send(()).unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    println!(
        "during the drain delay, /readyz -> {:?}",
        probe::probe(addr, "/readyz", wait).await
    );
    println!("in-flight request -> {:?}", in_flight.await.unwrap());
    println!("server: {:?}", server.await.unwrap().unwrap());
    println!(
        "after shutdown, /healthz -> {:?}\n",
        probe::probe(addr, "/healthz", wait).await
    );

    // 5. Build info.
    println!("build: {:?}\n", build_info::current());

    // 6. Lint the README's naive Dockerfile and ours.
    let naive = "FROM rust:1.75 as builder\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\n\nFROM debian:bookworm-slim\nCOPY --from=builder /app/target/release/app /usr/local/bin/\nCMD [\"app\"]\n";
    println!("the phase README's example Dockerfile:");
    for f in dockerfile_lint::lint(naive) {
        println!("  line {}: {:?} -- {}", f.line, f.rule, f.message);
    }
    for name in ["debian.Dockerfile", "scratch.Dockerfile"] {
        let text = std::fs::read_to_string(format!("{DOCKER_DIR}/{name}")).unwrap();
        println!(
            "docker/{name}: {} findings",
            dockerfile_lint::lint(&text).len()
        );
    }

    // 7. Compose start order.
    let yaml = std::fs::read_to_string(format!("{DOCKER_DIR}/compose.yaml")).unwrap();
    println!(
        "\ncompose start order: {:?}",
        compose::startup_order(&compose::parse(&yaml).unwrap())
    );
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("serve") => run_server().await,
        Some("healthcheck") => healthcheck(args.get(1).map_or("/readyz", String::as_str)).await,
        Some("demo") | Some("all") => {
            demo().await;
            0
        }
        Some("lint") if args.len() == 2 => match std::fs::read_to_string(&args[1]) {
            Ok(text) => {
                let findings = dockerfile_lint::lint(&text);
                for f in &findings {
                    println!("{}:{}: {:?}: {}", args[1], f.line, f.rule, f.message);
                }
                i32::from(!findings.is_empty())
            }
            Err(e) => {
                eprintln!("{}: {e}", args[1]);
                2
            }
        },
        _ => {
            println!("21-containerization -- reference solution\n");
            println!(
                "  cargo run -p m21-containerization-solution -- demo              config, probes, graceful shutdown, lint, compose"
            );
            println!(
                "  cargo run -p m21-containerization-solution -- serve             the service (PORT, LOG_LEVEL, ... from the environment)"
            );
            println!(
                "  cargo run -p m21-containerization-solution -- healthcheck [path] probe 127.0.0.1:$PORT (default /readyz); exit 0 or 1"
            );
            println!("  cargo run -p m21-containerization-solution -- lint <Dockerfile>");
            0
        }
    };
    std::process::exit(code);
}
