//! Exercise 3: the service, and shutting it down gracefully.
//!
//! When an orchestrator stops a container it sends SIGTERM, waits for the
//! grace period, then SIGKILLs. Exiting immediately on SIGTERM drops every
//! in-flight request. A graceful shutdown:
//!
//! 1. fails readiness, so load balancers stop sending new requests;
//! 2. keeps serving for a *drain delay* -- load balancers notice the change
//!    only on their next check, so requests keep arriving for a while;
//! 3. stops accepting connections and lets in-flight requests finish, up
//!    to a timeout;
//! 4. exits.

use crate::build_info;
use crate::config::Config;
use crate::health::{Health, Probe};
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::Notify;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub health: Arc<Health>,
}

impl AppState {
    pub fn new(config: Config) -> AppState {
        AppState {
            config: Arc::new(config),
            health: Arc::new(Health::new()),
        }
    }
}

async fn index() -> Json<serde_json::Value> {
    // In Docker and Kubernetes, HOSTNAME is the container / pod name: handy
    // for seeing which replica answered.
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".into());
    Json(serde_json::json!({ "message": "hello from a container", "hostname": hostname }))
}

#[derive(Deserialize)]
struct SlowQuery {
    ms: u64,
}

/// A request that takes a while: something to be in flight during shutdown.
async fn slow(Query(q): Query<SlowQuery>) -> String {
    tokio::time::sleep(Duration::from_millis(q.ms)).await;
    format!("done after {} ms", q.ms)
}

async fn healthz(State(s): State<AppState>) -> Probe {
    todo!("Exercise 3")
}

async fn readyz(State(s): State<AppState>) -> Probe {
    todo!("Exercise 3")
}

async fn startupz(State(s): State<AppState>) -> Probe {
    todo!("Exercise 3")
}

async fn version() -> Json<build_info::BuildInfo> {
    todo!("Exercise 3")
}

pub fn app(state: AppState) -> Router {
    todo!("Exercise 3")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownReport {
    /// false if the shutdown timeout cut in-flight requests off.
    pub drained_cleanly: bool,
    /// From the shutdown trigger to the end.
    pub elapsed: Duration,
}

/// Serves until `shutdown` completes, then shuts down gracefully (see the
/// module docs). Marks the service as started once it's listening.
pub async fn serve(
    listener: TcpListener,
    state: AppState,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<ShutdownReport> {
    todo!("Exercise 3")
}

/// Completes on Ctrl-C or SIGTERM -- what `docker stop` and Kubernetes send.
///
/// The process must *receive* SIGTERM to handle it: run the binary as PID 1
/// with the exec form, `ENTRYPOINT ["/app"]`. The shell form
/// (`ENTRYPOINT /app`) makes `/bin/sh` PID 1, which doesn't forward signals.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
