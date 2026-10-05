//! Exercise 4: retries.
//!
//! Retrying turns a blip into a success -- or a small outage into a big one,
//! when thousands of clients retry at once. The rules:
//!
//! - retry only errors that can succeed next time (`Unavailable`), never
//!   `InvalidArgument` or `NotFound`;
//! - retry only **idempotent** operations (Reserve is, thanks to its
//!   reservation id);
//! - back off exponentially, with **jitter** so clients don't retry in sync;
//! - cap the attempts.

use rand::Rng;
use std::future::Future;
use std::time::Duration;
use tonic::{Code, Status};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Total attempts, including the first.
    pub max_attempts: u32,
    pub base: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> RetryPolicy {
        todo!("Exercise 4")
    }
}

pub fn is_retryable(code: Code) -> bool {
    todo!("Exercise 4")
}

/// "Full jitter": a random delay between zero and the exponential cap,
/// `min(max_delay, base * 2^attempt)`. `attempt` starts at 0.
pub fn backoff(policy: &RetryPolicy, attempt: u32, rng: &mut impl Rng) -> Duration {
    todo!("Exercise 4")
}

/// Calls `op(attempt)` until it succeeds, fails with a non-retryable code,
/// or runs out of attempts. Returns the last error.
pub async fn retry<T, F, Fut>(policy: &RetryPolicy, mut op: F) -> Result<T, Status>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, Status>>,
{
    todo!("Exercise 4")
}
