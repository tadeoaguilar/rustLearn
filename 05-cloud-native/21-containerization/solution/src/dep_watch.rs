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
        Hysteresis {
            failures: failures.max(1),
            successes: successes.max(1),
            up: false,
            streak: 0,
        }
    }

    pub fn is_up(&self) -> bool {
        self.up
    }

    /// Records one check; returns the new state if it changed.
    pub fn observe(&mut self, ok: bool) -> Option<bool> {
        if ok == self.up {
            self.streak = 0; // the current state confirmed; a pending change resets
            return None;
        }
        self.streak += 1;
        let needed = if ok { self.successes } else { self.failures };
        if self.streak >= needed {
            self.up = ok;
            self.streak = 0;
            Some(ok)
        } else {
            None
        }
    }
}

/// "postgres://user:pw@db:5432/app" -> "db:5432". None if there's no port.
pub fn host_port(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split(['/', '?']).next()?;
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, hp)| hp);
    let (host, port) = host_port.rsplit_once(':')?;
    port.parse::<u16>().ok()?;
    (!host.is_empty()).then(|| host_port.to_string())
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
    let mut state = Hysteresis::new(config.failures, config.successes);
    health.set_dependency(&name, false);
    let mut ticker = tokio::time::interval(config.interval);
    loop {
        ticker.tick().await;
        let ok = matches!(
            tokio::time::timeout(config.timeout, tokio::net::TcpStream::connect(&addr)).await,
            Ok(Ok(_))
        );
        if let Some(up) = state.observe(ok) {
            health.set_dependency(&name, up);
        }
    }
}
