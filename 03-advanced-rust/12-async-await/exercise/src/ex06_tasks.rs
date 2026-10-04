//! Exercise 6: Spawning Tasks.
//!
//! `tokio::spawn` hands a future to the runtime to run *independently*, maybe
//! on another thread -- which is why it needs `Send + 'static`. It returns a
//! JoinHandle you can await for the result, or abort.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Semaphore, oneshot};
use tokio::task::{JoinError, JoinSet};
use tokio::time::{Duration, sleep};

pub async fn compute(n: u64) -> u64 {
    todo!("Exercise 6")
}

/// The exercise's program.
pub async fn spawn_ten() -> Vec<Result<u64, JoinError>> {
    todo!("Exercise 6")
}

/// Task 1a: cancellation by abort. The task stops at its next `.await`.
/// Awaiting the handle then gives `Err(JoinError)` with `is_cancelled()`.
pub async fn abort_a_task() -> bool {
    todo!("Exercise 6")
}

/// Task 1b: cooperative cancellation -- the task listens for a shutdown
/// signal and gets the chance to clean up. Returns how many ticks it did.
pub async fn cancel_cooperatively(run_for: Duration) -> u32 {
    todo!("Exercise 6")
}

/// Task 2: JoinSet -- a group of tasks; take results as they finish (not in
/// spawn order). Dropping the set aborts whatever is still running.
pub async fn joinset_sum(n: u64) -> u64 {
    todo!("Exercise 6")
}

/// Task 3: at most `limit` tasks run the guarded section at once. Returns the
/// highest concurrency actually observed.
pub async fn limited_concurrency(tasks: usize, limit: usize) -> usize {
    todo!("Exercise 6")
}

/// Real CPU-bound work must not run on the async worker threads -- it would
/// starve every other task. `spawn_blocking` moves it to a separate pool.
pub async fn cpu_heavy_sum(n: u64) -> u64 {
    todo!("Exercise 6")
}

pub async fn run() {
    todo!("Exercise 6")
}
