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
        todo!("Exercise 2")
    }
}

impl Health {
    pub fn new() -> Health {
        Health::default()
    }

    pub fn mark_started(&self) {
        todo!("Exercise 2")
    }

    pub fn begin_shutdown(&self) {
        todo!("Exercise 2")
    }

    pub fn is_shutting_down(&self) -> bool {
        todo!("Exercise 2")
    }

    /// Simulates a stuck process (a deadlock, an exhausted pool that never recovers).
    pub fn set_wedged(&self, wedged: bool) {
        todo!("Exercise 2")
    }

    /// Registers or updates a dependency's state.
    pub fn set_dependency(&self, name: &str, up: bool) {
        todo!("Exercise 2")
    }

    pub fn liveness(&self) -> Probe {
        todo!("Exercise 2")
    }

    pub fn startup(&self) -> Probe {
        todo!("Exercise 2")
    }

    pub fn readiness(&self) -> Probe {
        todo!("Exercise 2")
    }
}
