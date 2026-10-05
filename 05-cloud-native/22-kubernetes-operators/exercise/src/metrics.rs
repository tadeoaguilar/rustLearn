//! Exercise 7: retries and metrics.
//!
//! Reconciles fail -- the API server is restarting, a provider times out.
//! The controller retries, backing off exponentially *per object*, so one
//! broken object doesn't slow down the rest. And an operator nobody can
//! observe is one nobody trusts: count reconciles and their durations, and
//! expose them for Prometheus.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::sync::Mutex;
use std::time::Duration;

/// Per-object exponential backoff: base, 2x base, 4x base... capped.
#[derive(Debug)]
pub struct Backoff {
    base: Duration,
    max: Duration,
    failures: HashMap<String, u32>,
}

impl Backoff {
    pub fn new(base: Duration, max: Duration) -> Backoff {
        todo!("Exercise 7")
    }

    /// Records a failure of `key` and returns how long to wait before retrying.
    pub fn failure(&mut self, key: &str) -> Duration {
        todo!("Exercise 7")
    }

    /// A success resets the object's backoff.
    pub fn success(&mut self, key: &str) {
        todo!("Exercise 7")
    }

    pub fn failures(&self, key: &str) -> u32 {
        todo!("Exercise 7")
    }
}

#[derive(Debug, Default)]
struct Counters {
    reconciles: BTreeMap<(String, &'static str), u64>,
    duration_sum: BTreeMap<String, f64>,
    duration_count: BTreeMap<String, u64>,
}

#[derive(Debug, Default)]
pub struct Metrics {
    inner: Mutex<Counters>,
}

impl Metrics {
    pub fn new() -> Metrics {
        todo!("Exercise 7")
    }

    pub fn record(&self, kind: &str, ok: bool, elapsed: Duration) {
        todo!("Exercise 7")
    }

    pub fn reconciles(&self, kind: &str, ok: bool) -> u64 {
        todo!("Exercise 7")
    }

    /// The Prometheus text exposition format.
    pub fn render(&self) -> String {
        todo!("Exercise 7")
    }
}
