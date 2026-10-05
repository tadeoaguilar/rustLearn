//! Exercise 3: timeouts and retries -- with a budget.
//!
//! Mesh-level retries are powerful (every service gets them, no code) and
//! dangerous (every service gets them). A **retry budget** caps retries to a
//! fraction of recent requests: during a real outage, when every request
//! fails, the budget runs out and the proxy stops multiplying the load.
//! (Linkerd's default: retries may add 20% to the request rate, plus 10/s.)

use axum::http::Method;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Status(u16),
    ConnectFailed,
    TimedOut,
}

/// Only idempotent methods: a retried POST may create the order twice.
pub fn is_idempotent(method: &Method) -> bool {
    todo!("Exercise 3")
}

/// Worth retrying on another attempt (another endpoint, maybe).
pub fn is_retryable(method: &Method, outcome: Outcome) -> bool {
    todo!("Exercise 3")
}

#[derive(Debug)]
pub struct RetryBudget {
    ratio: f64,
    min_per_window: u32,
    window: Duration,
    requests: VecDeque<Instant>,
    retries: VecDeque<Instant>,
}

impl RetryBudget {
    /// Retries may add `ratio` of the requests in the last `window`, plus
    /// `min_per_window` (so low-traffic services can still retry).
    pub fn new(ratio: f64, min_per_window: u32, window: Duration) -> RetryBudget {
        todo!("Exercise 3")
    }

    fn expire(&mut self, now: Instant) {
        todo!("Exercise 3")
    }

    /// Call for every original (non-retry) request.
    pub fn record_request(&mut self, now: Instant) {
        todo!("Exercise 3")
    }

    /// May we retry now? If so, the retry is counted.
    pub fn try_retry(&mut self, now: Instant) -> bool {
        todo!("Exercise 3")
    }
}
