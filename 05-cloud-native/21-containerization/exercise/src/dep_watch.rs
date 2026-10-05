//! Bonus: readiness that follows a dependency.
//!
//! The service is only useful while its database is reachable, so readiness
//! should follow it -- but one dropped connection shouldn't flap the replica
//! out of the load balancer. Kubernetes probes have the same knobs:
//! `failureThreshold` and `successThreshold`.

use crate::health::Health;
use std::sync::Arc;
use std::time::Duration;

/// Changes state only after `failures` consecutive failures (going down) or
/// `successes` consecutive successes (coming back up). Starts down.
#[derive(Debug, Clone)]
pub struct Hysteresis {
    failures: u32,
    successes: u32,
    up: bool,
    streak: u32,
}

impl Hysteresis {
    pub fn new(failures: u32, successes: u32) -> Hysteresis {
        todo!("Bonus")
    }

    pub fn is_up(&self) -> bool {
        todo!("Bonus")
    }

    /// Records one check; returns the new state if it changed.
    pub fn observe(&mut self, ok: bool) -> Option<bool> {
        todo!("Bonus")
    }
}

/// "postgres://user:pw@db:5432/app" -> "db:5432". None if there's no port.
pub fn host_port(url: &str) -> Option<String> {
    todo!("Bonus")
}

#[derive(Debug, Clone, Copy)]
pub struct WatchConfig {
    pub interval: Duration,
    pub timeout: Duration,
    pub failures: u32,
    pub successes: u32,
}

/// Forever: try a TCP connection to `addr` every `interval` and record the
/// smoothed result as dependency `name`. Spawn it as a task.
pub async fn watch_tcp(name: String, addr: String, health: Arc<Health>, config: WatchConfig) {
    todo!("Bonus")
}
