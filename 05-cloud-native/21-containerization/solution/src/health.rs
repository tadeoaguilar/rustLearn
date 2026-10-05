//! Exercise 2: liveness, readiness and startup.
//!
//! Orchestrators ask three different questions, and mixing them up causes
//! outages:
//!
//! - **liveness** -- "is this process wedged?" Failing it gets the container
//!   *restarted*. It must not depend on other services: if the database goes
//!   down and every replica fails liveness, everything restarts in a loop and
//!   nothing gets better.
//! - **readiness** -- "should this replica get traffic right now?" Failing it
//!   only takes the replica out of the load balancer. Dependencies belong
//!   here, and so does "I'm shutting down".
//! - **startup** -- "has it finished starting?" Until it passes, the other
//!   probes are not run, so slow starts don't count as failures.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default)]
pub struct Health {
    started: AtomicBool,
    shutting_down: AtomicBool,
    wedged: AtomicBool,
    dependencies: Mutex<BTreeMap<String, bool>>,
}

/// A probe's answer: 200 if `ok`, otherwise 503, with the details as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Probe {
    pub ok: bool,
    pub checks: BTreeMap<String, String>,
}

impl IntoResponse for Probe {
    fn into_response(self) -> Response {
        let status = if self.ok {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };
        (status, Json(self)).into_response()
    }
}

impl Health {
    pub fn new() -> Health {
        Health::default()
    }

    pub fn mark_started(&self) {
        self.started.store(true, Ordering::SeqCst);
    }

    pub fn begin_shutdown(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    /// Simulates a stuck process (a deadlock, an exhausted pool that never recovers).
    pub fn set_wedged(&self, wedged: bool) {
        self.wedged.store(wedged, Ordering::SeqCst);
    }

    /// Registers or updates a dependency's state.
    pub fn set_dependency(&self, name: &str, up: bool) {
        self.dependencies
            .lock()
            .unwrap()
            .insert(name.to_string(), up);
    }

    pub fn liveness(&self) -> Probe {
        let wedged = self.wedged.load(Ordering::SeqCst);
        let checks = BTreeMap::from([(
            "process".to_string(),
            if wedged { "wedged" } else { "ok" }.to_string(),
        )]);
        Probe {
            ok: !wedged,
            checks,
        }
    }

    pub fn startup(&self) -> Probe {
        let started = self.started.load(Ordering::SeqCst);
        let checks = BTreeMap::from([(
            "startup".to_string(),
            if started { "ok" } else { "starting" }.to_string(),
        )]);
        Probe {
            ok: started,
            checks,
        }
    }

    pub fn readiness(&self) -> Probe {
        let mut checks = BTreeMap::new();
        let started = self.started.load(Ordering::SeqCst);
        let stopping = self.is_shutting_down();
        checks.insert(
            "startup".to_string(),
            if started { "ok" } else { "starting" }.to_string(),
        );
        checks.insert(
            "shutdown".to_string(),
            if stopping { "shutting down" } else { "ok" }.to_string(),
        );
        let mut deps_ok = true;
        for (name, up) in self.dependencies.lock().unwrap().iter() {
            deps_ok &= *up;
            checks.insert(name.clone(), if *up { "ok" } else { "down" }.to_string());
        }
        Probe {
            ok: started && !stopping && deps_ok,
            checks,
        }
    }
}
