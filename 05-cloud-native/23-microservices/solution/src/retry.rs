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
        RetryPolicy {
            max_attempts: 4,
            base: Duration::from_millis(50),
            max_delay: Duration::from_secs(2),
        }
    }
}

pub fn is_retryable(code: Code) -> bool {
    matches!(
        code,
        Code::Unavailable | Code::ResourceExhausted | Code::Aborted
    )
}

/// "Full jitter": a random delay between zero and the exponential cap,
/// `min(max_delay, base * 2^attempt)`. `attempt` starts at 0.
pub fn backoff(policy: &RetryPolicy, attempt: u32, rng: &mut impl Rng) -> Duration {
    let cap = policy
        .base
        .saturating_mul(2u32.saturating_pow(attempt))
        .min(policy.max_delay);
    let nanos = u64::try_from(cap.as_nanos()).unwrap_or(u64::MAX);
    Duration::from_nanos(rng.gen_range(0..=nanos))
}

/// Calls `op(attempt)` until it succeeds, fails with a non-retryable code,
/// or runs out of attempts. Returns the last error.
pub async fn retry<T, F, Fut>(policy: &RetryPolicy, mut op: F) -> Result<T, Status>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, Status>>,
{
    let mut attempt = 0;
    loop {
        match op(attempt).await {
            Ok(value) => return Ok(value),
            Err(status) if is_retryable(status.code()) && attempt + 1 < policy.max_attempts => {
                // (One statement: the thread-local RNG must not live across the await.)
                let delay = backoff(policy, attempt, &mut rand::thread_rng());
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(status) => return Err(status),
        }
    }
}
