//! Exercise 2: Concurrent Execution.
//!
//! Concurrency is not parallelism: `join!` runs futures *interleaved* on one
//! task. While one waits on its timer, the other makes progress. Total time
//! is the longest wait, not the sum.

use futures::future::join_all;
use std::future::Future;
use tokio::time::{Duration, Instant, sleep, timeout};

pub async fn task_one() -> String {
    todo!("Exercise 2")
}

pub async fn task_two() -> String {
    todo!("Exercise 2")
}

/// `tokio::time::Instant`, not `std::time::Instant`: in tests with a paused
/// clock it measures *virtual* time, so these run instantly and exactly.
pub async fn sequential() -> (Vec<String>, Duration) {
    todo!("Exercise 2")
}

pub async fn concurrent() -> (Vec<String>, Duration) {
    todo!("Exercise 2")
}

async fn numbered_task(i: u64) -> u64 {
    todo!("Exercise 2")
}

/// Task 1: ten at once. `join!` needs every future written out, so for a
/// dynamic number use `join_all`. Results come back in *input* order,
/// whatever order they finished in.
pub async fn ten_concurrently() -> (Vec<u64>, Duration) {
    todo!("Exercise 2")
}

/// Task 2: `select!` -- the first branch to complete wins; the others are
/// *dropped*, which cancels them.
pub async fn first_to_finish() -> &'static str {
    todo!("Exercise 2")
}

/// Task 3: `timeout` wraps any future; Err(Elapsed) if it's too slow.
pub async fn with_timeout<F: Future>(limit: Duration, fut: F) -> Option<F::Output> {
    todo!("Exercise 2")
}

pub async fn run() {
    todo!("Exercise 2")
}
