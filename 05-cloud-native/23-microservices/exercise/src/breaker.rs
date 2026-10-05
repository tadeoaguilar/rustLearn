//! Exercise 5: a circuit breaker.
//!
//! When a dependency is down, every call waits for a timeout and fails --
//! tying up threads and connections, and hammering the dependency as it tries
//! to recover. A breaker notices consecutive failures and **opens**: calls
//! fail immediately for a while. Then it lets **one** probe through
//! (half-open): success closes it, failure opens it again.
//!
//! ```text
//!   Closed --N failures--> Open --cool-down--> HalfOpen --probe ok--> Closed
//!                           ^                      |
//!                           +-----probe failed-----+
//! ```
//!
//! Time is passed in (`now`), so tests don't sleep.

use std::time::{Duration, Instant};
use tonic::Code;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    open_for: Duration,
    consecutive_failures: u32,
    opened_at: Option<Instant>,
    /// When the current half-open probe started. A caller that gives up
    /// (deadline, cancellation) never reports back, so a probe older than
    /// `open_for` is treated as lost and another one is allowed.
    probe_started: Option<Instant>,
}

/// Only failures that say "the dependency is unhealthy" count. A NotFound
/// is the dependency working correctly.
pub fn counts_as_failure(code: Code) -> bool {
    todo!("Exercise 5")
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, open_for: Duration) -> CircuitBreaker {
        todo!("Exercise 5")
    }

    pub fn state(&self, now: Instant) -> State {
        todo!("Exercise 5")
    }

    /// May a call go through? In half-open state, only one probe at a time.
    pub fn try_acquire(&mut self, now: Instant) -> bool {
        todo!("Exercise 5")
    }

    pub fn on_success(&mut self) {
        todo!("Exercise 5")
    }

    pub fn on_failure(&mut self, now: Instant) {
        todo!("Exercise 5")
    }
}
