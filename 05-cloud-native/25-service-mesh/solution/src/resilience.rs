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
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::PUT | Method::DELETE
    )
}

/// Worth retrying on another attempt (another endpoint, maybe).
pub fn is_retryable(method: &Method, outcome: Outcome) -> bool {
    is_idempotent(method)
        && match outcome {
            Outcome::ConnectFailed => true,
            Outcome::Status(s) => matches!(s, 502 | 503),
            // A timed-out request may still be running upstream: don't pile on.
            Outcome::TimedOut => false,
        }
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
        RetryBudget {
            ratio,
            min_per_window,
            window,
            requests: VecDeque::new(),
            retries: VecDeque::new(),
        }
    }

    fn expire(&mut self, now: Instant) {
        for q in [&mut self.requests, &mut self.retries] {
            while q
                .front()
                .is_some_and(|t| now.duration_since(*t) >= self.window)
            {
                q.pop_front();
            }
        }
    }

    /// Call for every original (non-retry) request.
    pub fn record_request(&mut self, now: Instant) {
        self.expire(now);
        self.requests.push_back(now);
    }

    /// May we retry now? If so, the retry is counted.
    pub fn try_retry(&mut self, now: Instant) -> bool {
        self.expire(now);
        let allowed = self.min_per_window as f64 + self.ratio * self.requests.len() as f64;
        if (self.retries.len() as f64) < allowed.floor() {
            self.retries.push_back(now);
            true
        } else {
            false
        }
    }
}
