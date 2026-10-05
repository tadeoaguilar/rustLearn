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
        Backoff {
            base,
            max,
            failures: HashMap::new(),
        }
    }

    /// Records a failure of `key` and returns how long to wait before retrying.
    pub fn failure(&mut self, key: &str) -> Duration {
        let n = self.failures.entry(key.to_string()).or_insert(0);
        let delay = self
            .base
            .saturating_mul(2u32.saturating_pow(*n))
            .min(self.max);
        *n = n.saturating_add(1);
        delay
    }

    /// A success resets the object's backoff.
    pub fn success(&mut self, key: &str) {
        self.failures.remove(key);
    }

    pub fn failures(&self, key: &str) -> u32 {
        self.failures.get(key).copied().unwrap_or(0)
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
        Metrics::default()
    }

    pub fn record(&self, kind: &str, ok: bool, elapsed: Duration) {
        let mut c = self.inner.lock().unwrap();
        *c.reconciles
            .entry((kind.to_string(), if ok { "ok" } else { "error" }))
            .or_default() += 1;
        *c.duration_sum.entry(kind.to_string()).or_default() += elapsed.as_secs_f64();
        *c.duration_count.entry(kind.to_string()).or_default() += 1;
    }

    pub fn reconciles(&self, kind: &str, ok: bool) -> u64 {
        let c = self.inner.lock().unwrap();
        c.reconciles
            .get(&(kind.to_string(), if ok { "ok" } else { "error" }))
            .copied()
            .unwrap_or(0)
    }

    /// The Prometheus text exposition format.
    pub fn render(&self) -> String {
        let c = self.inner.lock().unwrap();
        let mut out = String::new();
        out.push_str(
            "# HELP rustlearn_operator_reconciles_total Reconciliations by kind and result.\n",
        );
        out.push_str("# TYPE rustlearn_operator_reconciles_total counter\n");
        for ((kind, result), n) in &c.reconciles {
            let _ = writeln!(
                out,
                "rustlearn_operator_reconciles_total{{kind=\"{kind}\",result=\"{result}\"}} {n}"
            );
        }
        out.push_str(
            "# HELP rustlearn_operator_reconcile_duration_seconds Time spent reconciling.\n",
        );
        out.push_str("# TYPE rustlearn_operator_reconcile_duration_seconds summary\n");
        for (kind, sum) in &c.duration_sum {
            let _ = writeln!(
                out,
                "rustlearn_operator_reconcile_duration_seconds_sum{{kind=\"{kind}\"}} {sum}"
            );
            let _ = writeln!(
                out,
                "rustlearn_operator_reconcile_duration_seconds_count{{kind=\"{kind}\"}} {}",
                c.duration_count[kind]
            );
        }
        out
    }
}
