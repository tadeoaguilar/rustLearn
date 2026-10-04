//! Exercise 2: Concurrent Execution.
//!
//! Concurrency is not parallelism: `join!` runs futures *interleaved* on one
//! task. While one waits on its timer, the other makes progress. Total time
//! is the longest wait, not the sum.

use futures::future::join_all;
use std::future::Future;
use tokio::time::{Duration, Instant, sleep, timeout};

pub async fn task_one() -> String {
    sleep(Duration::from_secs(1)).await;
    "Task 1 complete".to_string()
}

pub async fn task_two() -> String {
    sleep(Duration::from_secs(2)).await;
    "Task 2 complete".to_string()
}

/// `tokio::time::Instant`, not `std::time::Instant`: in tests with a paused
/// clock it measures *virtual* time, so these run instantly and exactly.
pub async fn sequential() -> (Vec<String>, Duration) {
    let start = Instant::now();
    let r1 = task_one().await;
    let r2 = task_two().await;
    (vec![r1, r2], start.elapsed()) // ~3s
}

pub async fn concurrent() -> (Vec<String>, Duration) {
    let start = Instant::now();
    let (r1, r2) = tokio::join!(task_one(), task_two());
    (vec![r1, r2], start.elapsed()) // ~2s
}

async fn numbered_task(i: u64) -> u64 {
    sleep(Duration::from_millis(100 * (10 - i))).await; // later ones finish first
    i
}

/// Task 1: ten at once. `join!` needs every future written out, so for a
/// dynamic number use `join_all`. Results come back in *input* order,
/// whatever order they finished in.
pub async fn ten_concurrently() -> (Vec<u64>, Duration) {
    let start = Instant::now();
    let results = join_all((0..10).map(numbered_task)).await;
    (results, start.elapsed()) // ~1s, not 5.5s
}

/// Task 2: `select!` -- the first branch to complete wins; the others are
/// *dropped*, which cancels them.
pub async fn first_to_finish() -> &'static str {
    tokio::select! {
        r = task_one() => { let _ = r; "task one" }
        r = task_two() => { let _ = r; "task two" }
    }
}

/// Task 3: `timeout` wraps any future; Err(Elapsed) if it's too slow.
pub async fn with_timeout<F: Future>(limit: Duration, fut: F) -> Option<F::Output> {
    timeout(limit, fut).await.ok()
}

pub async fn run() {
    let (_, t) = sequential().await;
    println!("Sequential: {t:.1?}");
    let (results, t) = concurrent().await;
    println!("Concurrent: {t:.1?} -> {results:?}");
    let (order, t) = ten_concurrently().await;
    println!("10 tasks via join_all: {t:.1?}, results in input order: {order:?}");
    println!("select! winner: {}", first_to_finish().await);
    println!(
        "task_two within 500ms? {:?}",
        with_timeout(Duration::from_millis(500), task_two()).await
    );
    println!(
        "task_one within 1.5s?  {:?}",
        with_timeout(Duration::from_millis(1500), task_one()).await
    );
}
