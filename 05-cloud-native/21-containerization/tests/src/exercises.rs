use crate::sut::*;
use config::{Config, LogLevel};
use dockerfile_lint::Rule;
use server::AppState;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |k| map.get(k).cloned()
}

// ---- Exercise 1: configuration -------------------------------------------------------------

#[test]
fn ex1_defaults() {
    let c = Config::from_lookup(lookup(&[])).unwrap();
    assert_eq!(
        c.bind_addr().to_string(),
        "0.0.0.0:8080",
        "containers must listen on all interfaces"
    );
    assert_eq!(c.log_level, LogLevel::Info);
    assert_eq!(c.api_token, None);
    assert_eq!(c.drain_delay, Duration::from_secs(5));
}

#[test]
fn ex1_values_and_empty_means_unset() {
    let c = Config::from_lookup(lookup(&[
        ("PORT", "9000"),
        ("LOG_LEVEL", "DEBUG"),
        ("HOST", ""),
        ("DATABASE_URL", "postgres://db/app"),
    ]))
    .unwrap();
    assert_eq!(c.port, 9000);
    assert_eq!(c.log_level, LogLevel::Debug);
    assert_eq!(c.host.to_string(), "0.0.0.0");
    assert_eq!(c.database_url.as_deref(), Some("postgres://db/app"));
}

#[test]
fn ex1_reports_every_error_at_once() {
    let errors = Config::from_lookup(lookup(&[
        ("PORT", "eighty"),
        ("LOG_LEVEL", "loud"),
        ("DRAIN_DELAY_MS", "-1"),
    ]))
    .unwrap_err();
    let vars: Vec<&str> = errors.0.iter().map(|e| e.var.as_str()).collect();
    assert_eq!(vars, ["PORT", "LOG_LEVEL", "DRAIN_DELAY_MS"]);
    assert!(errors.to_string().contains("PORT"));
    assert!(Config::from_lookup(lookup(&[("PORT", "0")])).is_err());
}

#[test]
fn ex1_secrets_from_files_and_never_printed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token");
    std::fs::write(&path, "s3cr3t\n").unwrap();
    let c = Config::from_lookup(lookup(&[("API_TOKEN_FILE", path.to_str().unwrap())])).unwrap();
    assert_eq!(
        c.api_token.as_ref().unwrap().expose(),
        "s3cr3t",
        "trailing newline removed"
    );
    assert!(
        !format!("{c:?}").contains("s3cr3t"),
        "Debug must redact secrets"
    );

    let both = Config::from_lookup(lookup(&[
        ("API_TOKEN", "a"),
        ("API_TOKEN_FILE", path.to_str().unwrap()),
    ]));
    assert!(both.is_err());
    let missing = Config::from_lookup(lookup(&[("API_TOKEN_FILE", "/no/such/file")])).unwrap_err();
    assert_eq!(missing.0[0].var, "API_TOKEN_FILE");
}

// ---- Exercise 2: probes ------------------------------------------------------------------------

#[test]
fn ex2_probe_semantics() {
    let h = health::Health::new();
    assert!(!h.startup().ok && !h.readiness().ok, "not started yet");
    assert!(h.liveness().ok, "starting is not the same as dead");
    h.mark_started();
    assert!(h.startup().ok && h.readiness().ok);

    h.set_dependency("database", false);
    let ready = h.readiness();
    assert!(!ready.ok);
    assert_eq!(
        ready.checks.get("database").map(String::as_str),
        Some("down")
    );
    assert!(
        h.liveness().ok,
        "a dependency outage must NOT fail liveness (restart loops)"
    );
    h.set_dependency("database", true);
    assert!(h.readiness().ok);

    h.set_wedged(true);
    assert!(!h.liveness().ok);
    h.set_wedged(false);
    h.begin_shutdown();
    assert!(!h.readiness().ok && h.liveness().ok);
}

// ---- Exercise 3: the server and graceful shutdown -------------------------------------------------

struct Running {
    addr: SocketAddr,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    server: tokio::task::JoinHandle<std::io::Result<server::ShutdownReport>>,
    state: AppState,
}

async fn start(drain_ms: u64, timeout_ms: u64) -> Running {
    let config = Config::from_lookup(lookup(&[])).unwrap();
    let config = Config {
        drain_delay: Duration::from_millis(drain_ms),
        shutdown_timeout: Duration::from_millis(timeout_ms),
        ..config
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let state = AppState::new(config);
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(server::serve(listener, state.clone(), async {
        let _ = rx.await;
    }));
    tokio::time::sleep(Duration::from_millis(30)).await;
    Running {
        addr,
        stop: Some(tx),
        server,
        state,
    }
}

async fn get(addr: SocketAddr, path: &str) -> Result<(u16, String), reqwest::Error> {
    // A fresh client per request: no pooled keep-alive connection survives into the next check.
    let r = reqwest::Client::new()
        .get(format!("http://{addr}{path}"))
        .send()
        .await?;
    Ok((r.status().as_u16(), r.text().await?))
}

#[tokio::test]
async fn ex3_routes_and_probes() {
    let s = start(0, 1000).await;
    assert_eq!(get(s.addr, "/healthz").await.unwrap().0, 200);
    assert_eq!(
        get(s.addr, "/startupz").await.unwrap().0,
        200,
        "serve() marks the service started"
    );
    s.state.health.set_dependency("database", false);
    let (status, body) = get(s.addr, "/readyz").await.unwrap();
    assert_eq!(status, 503);
    assert!(body.contains("\"database\":\"down\""), "{body}");
    let (status, body) = get(s.addr, "/version").await.unwrap();
    assert_eq!(status, 200);
    assert!(body.contains("\"version\""), "{body}");
}

#[tokio::test]
async fn ex3_graceful_shutdown_drains_in_flight_requests() {
    let mut s = start(200, 2000).await;
    let addr = s.addr;
    let in_flight = tokio::spawn(async move { get(addr, "/slow?ms=400").await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    s.stop.take().unwrap().send(()).unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Draining: still serving, but no longer ready.
    assert_eq!(get(addr, "/readyz").await.unwrap().0, 503);
    assert_eq!(get(addr, "/healthz").await.unwrap().0, 200);

    assert_eq!(
        in_flight.await.unwrap().unwrap(),
        (200, "done after 400 ms".to_string()),
        "in-flight request completes"
    );
    let report = s.server.await.unwrap().unwrap();
    assert!(report.drained_cleanly);
    assert!(
        report.elapsed >= Duration::from_millis(200),
        "waited for the drain delay: {report:?}"
    );
    assert!(get(addr, "/healthz").await.is_err(), "no longer listening");
}

#[tokio::test]
async fn ex3_shutdown_timeout_cuts_off_stuck_requests() {
    let mut s = start(0, 200).await;
    let addr = s.addr;
    let stuck = tokio::spawn(async move { get(addr, "/slow?ms=5000").await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    let t = Instant::now();
    s.stop.take().unwrap().send(()).unwrap();
    let report = s.server.await.unwrap().unwrap();
    assert!(!report.drained_cleanly);
    assert!(
        t.elapsed() < Duration::from_secs(2),
        "gave up after the timeout, not after 5 s"
    );
    stuck.abort();
}

// ---- Exercise 4: the built-in health check -------------------------------------------------------

#[tokio::test]
async fn ex4_probe_reports_status() {
    let s = start(0, 1000).await;
    let wait = Duration::from_secs(1);
    assert_eq!(probe::probe(s.addr, "/healthz", wait).await, Ok(200));
    s.state.health.set_dependency("db", false);
    let unhealthy = probe::probe(s.addr, "/readyz", wait).await;
    assert_eq!(unhealthy, Err(probe::ProbeError::Unhealthy(503)));
    assert_eq!(probe::exit_code(&unhealthy), 1);
    assert_eq!(probe::exit_code(&Ok(200)), 0);
}

#[tokio::test]
async fn ex4_probe_fails_fast_on_dead_and_hung_servers() {
    // Nothing listening.
    let free = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = free.local_addr().unwrap();
    drop(free);
    assert!(matches!(
        probe::probe(addr, "/healthz", Duration::from_secs(1)).await,
        Err(probe::ProbeError::Connect(_))
    ));

    // Accepts, never answers.
    let hung = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = hung.local_addr().unwrap();
    let _keep = tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((sock, _)) = hung.accept().await {
            held.push(sock);
        }
    });
    let t = Instant::now();
    assert_eq!(
        probe::probe(addr, "/healthz", Duration::from_millis(200)).await,
        Err(probe::ProbeError::Timeout)
    );
    assert!(t.elapsed() < Duration::from_secs(1));

    // Speaks something that isn't HTTP.
    let liar = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = liar.local_addr().unwrap();
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let (mut sock, _) = liar.accept().await.unwrap();
        sock.write_all(b"SSH-2.0-OpenSSH\r\n").await.unwrap();
    });
    assert!(matches!(
        probe::probe(addr, "/", Duration::from_secs(1)).await,
        Err(probe::ProbeError::BadResponse(_))
    ));
}

// ---- Exercise 5: build info --------------------------------------------------------------------------

#[test]
fn ex5_build_info() {
    let info = build_info::current();
    assert!(info.name.starts_with("m21-containerization"));
    assert_eq!(info.version, "0.1.0");
    assert!(!info.git_sha.is_empty());
    assert_eq!(info.profile, "debug", "tests build in the dev profile");
    assert!(
        info.target.contains('-'),
        "a target triple: {}",
        info.target
    );
    assert_eq!(
        info.linkage,
        if cfg!(target_env = "musl") {
            "static"
        } else {
            "dynamic"
        }
    );
}

// ---- Exercise 6: Dockerfile linter -----------------------------------------------------------------

const NAIVE: &str = "\
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/
CMD [\"app\"]
";

fn rules(text: &str) -> Vec<(usize, Rule)> {
    dockerfile_lint::lint(text)
        .into_iter()
        .map(|f| (f.line, f.rule))
        .collect()
}

#[test]
fn ex6_parse_joins_continuations_and_skips_comments() {
    let text = "# syntax=docker/dockerfile:1\nFROM alpine:3.20\n\nRUN apk add \\\n    # a comment inside\n    curl \\\n    jq\nuser 10001\n";
    let parsed = dockerfile_lint::parse(text);
    let summary: Vec<(usize, &str, &str)> = parsed
        .iter()
        .map(|i| (i.line, i.keyword.as_str(), i.args.as_str()))
        .collect();
    assert_eq!(
        summary,
        [
            (2, "FROM", "alpine:3.20"),
            (4, "RUN", "apk add curl jq"),
            (8, "USER", "10001")
        ]
    );
}

#[test]
fn ex6_lints_the_phase_readme_example() {
    assert_eq!(
        rules(NAIVE),
        [
            (4, Rule::NoDependencyCaching),
            (6, Rule::RunsAsRoot),
            (6, Rule::NoHealthcheck)
        ]
    );
}

#[test]
fn ex6_each_rule() {
    use Rule::*;
    assert_eq!(
        rules("FROM rust\nUSER 1\nHEALTHCHECK NONE\n"),
        [(1, UnpinnedBaseImage), (1, SingleStage)]
    );
    assert!(
        rules("FROM rust:latest AS b\nFROM b\nUSER 1\nHEALTHCHECK NONE\n")
            .contains(&(1, UnpinnedBaseImage))
    );
    let ok = "FROM --platform=$BUILDPLATFORM localhost:5000/rust@sha256:abc AS build\nRUN cargo chef cook --release\nRUN cargo build --release\nFROM scratch\nCOPY --from=build /a /a\nUSER 10001:10001\nHEALTHCHECK CMD [\"/a\", \"healthcheck\"]\nENTRYPOINT [\"/a\"]\n";
    assert_eq!(
        rules(ok),
        [],
        "digest pins, scratch, stage references and cargo-chef are fine"
    );
    assert!(
        rules("FROM localhost:5000/rust AS b\nFROM b\n").contains(&(1, UnpinnedBaseImage)),
        "a registry port isn't a tag"
    );
    assert!(
        rules(
            "FROM a:1 AS b\nFROM scratch\nADD app /app\nUSER 1\nHEALTHCHECK NONE\nENTRYPOINT /app\n"
        )
        .ends_with(&[(3, AddForLocalFiles), (6, ShellFormCommand)])
    );
    assert!(!rules("FROM a:1 AS b\nFROM scratch\nADD https://example.com/x.tgz /x\nUSER 1\nHEALTHCHECK NONE\n").iter().any(|(_, r)| *r == AddForLocalFiles));
    assert!(
        rules("FROM a:1 AS b\nFROM scratch\nUSER root\nHEALTHCHECK NONE\n")
            .contains(&(3, RunsAsRoot))
    );
}

// ---- Exercise 7: Compose ---------------------------------------------------------------------------

#[test]
fn ex7_startup_waves() {
    let yaml = "services:\n  web:\n    depends_on: [api, cache]\n  api:\n    depends_on:\n      db: { condition: service_healthy }\n      cache: { condition: service_started }\n  cache: { image: redis:7 }\n  db:\n    image: postgres:17\n    healthcheck: { test: [CMD, pg_isready] }\n";
    let file = compose::parse(yaml).unwrap();
    assert_eq!(
        file.services["api"].depends_on()["db"],
        compose::Condition::ServiceHealthy
    );
    assert_eq!(
        file.services["web"].depends_on()["api"],
        compose::Condition::ServiceStarted
    );
    assert_eq!(
        compose::startup_order(&file).unwrap(),
        vec![vec!["cache", "db"], vec!["api"], vec!["web"]]
    );
}

#[test]
fn ex7_validation() {
    use compose::ComposeError;
    let check = |yaml: &str| compose::startup_order(&compose::parse(yaml).unwrap());
    assert_eq!(
        check("services:\n  a: { depends_on: [ghost] }\n"),
        Err(ComposeError::UnknownDependency {
            service: "a".into(),
            dependency: "ghost".into()
        })
    );
    assert_eq!(
        check(
            "services:\n  a:\n    depends_on: { b: { condition: service_healthy } }\n  b: { image: x }\n"
        ),
        Err(ComposeError::NoHealthcheck {
            service: "a".into(),
            dependency: "b".into()
        })
    );
    assert_eq!(
        check("services:\n  a: { depends_on: [b] }\n  b: { depends_on: [a] }\n  c: { image: x }\n"),
        Err(ComposeError::Cycle(vec!["a".into(), "b".into()]))
    );
    assert!(matches!(
        compose::parse("services: [not, a, map]"),
        Err(ComposeError::Parse(_))
    ));
}

// ---- Exercise 8: your Dockerfiles and compose.yaml --------------------------------------------------

fn docker_file(name: &str) -> String {
    std::fs::read_to_string(format!("{}/{name}", crate::sut::DOCKER_DIR)).unwrap()
}

#[test]
fn ex8_dockerfiles_pass_the_linter() {
    for name in ["debian.Dockerfile", "scratch.Dockerfile"] {
        let findings = dockerfile_lint::lint(&docker_file(name));
        assert!(findings.is_empty(), "docker/{name}: {findings:#?}");
    }
    let scratch = docker_file("scratch.Dockerfile");
    let final_from = dockerfile_lint::parse(&scratch)
        .into_iter()
        .rfind(|i| i.keyword == "FROM")
        .unwrap();
    assert_eq!(
        final_from.args, "scratch",
        "the scratch image's final stage is FROM scratch"
    );
}

#[test]
fn ex8_compose_starts_db_then_api_then_admin() {
    let file = compose::parse(&docker_file("compose.yaml")).unwrap();
    assert_eq!(
        compose::startup_order(&file).unwrap(),
        vec![vec!["db"], vec!["api"], vec!["admin"]]
    );
}

// ---- Bonus: dependency-aware readiness --------------------------------------------------------------

#[test]
fn bonus_hysteresis_and_urls() {
    let mut h = dep_watch::Hysteresis::new(3, 2);
    assert!(!h.is_up());
    assert_eq!(h.observe(true), None);
    assert_eq!(h.observe(true), Some(true), "up after 2 successes");
    assert_eq!(h.observe(false), None);
    assert_eq!(
        h.observe(true),
        None,
        "a success in between resets the failure streak"
    );
    assert_eq!(h.observe(false), None);
    assert_eq!(h.observe(false), None);
    assert_eq!(
        h.observe(false),
        Some(false),
        "down after 3 consecutive failures"
    );

    assert_eq!(
        dep_watch::host_port("postgres://user:p%40ss@db:5432/app?sslmode=disable").as_deref(),
        Some("db:5432")
    );
    assert_eq!(
        dep_watch::host_port("redis://cache:6379").as_deref(),
        Some("cache:6379")
    );
    assert_eq!(dep_watch::host_port("postgres://db/app"), None, "no port");
}

#[tokio::test]
async fn bonus_readiness_follows_a_tcp_dependency() {
    let db = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let db_addr = db.local_addr().unwrap().to_string();
    let accept = tokio::spawn(async move {
        loop {
            let _ = db.accept().await;
        }
    });
    let health = std::sync::Arc::new(health::Health::new());
    health.mark_started();
    let cfg = dep_watch::WatchConfig {
        interval: Duration::from_millis(20),
        timeout: Duration::from_millis(100),
        failures: 2,
        successes: 1,
    };
    let watcher = tokio::spawn(dep_watch::watch_tcp(
        "database".into(),
        db_addr,
        health.clone(),
        cfg,
    ));
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(health.readiness().ok, "database reachable");

    accept.abort(); // dropping the listener: connections are refused
    tokio::time::sleep(Duration::from_millis(200)).await;
    let r = health.readiness();
    assert!(!r.ok);
    assert_eq!(r.checks["database"], "down");
    watcher.abort();
}
