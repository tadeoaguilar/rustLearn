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
    sleep(Duration::from_millis(100)).await;
    n * n
}

/// The exercise's program.
pub async fn spawn_ten() -> Vec<Result<u64, JoinError>> {
    let handles: Vec<_> = (0..10).map(|i| tokio::spawn(compute(i))).collect();
    let mut results = Vec::new();
    for h in handles {
        results.push(h.await);
    }
    results
}

/// Task 1a: cancellation by abort. The task stops at its next `.await`.
/// Awaiting the handle then gives `Err(JoinError)` with `is_cancelled()`.
pub async fn abort_a_task() -> bool {
    let handle = tokio::spawn(async {
        sleep(Duration::from_secs(60)).await;
        "never"
    });
    sleep(Duration::from_millis(10)).await;
    handle.abort();
    matches!(handle.await, Err(e) if e.is_cancelled())
}

/// Task 1b: cooperative cancellation -- the task listens for a shutdown
/// signal and gets the chance to clean up. Returns how many ticks it did.
pub async fn cancel_cooperatively(run_for: Duration) -> u32 {
    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
    let worker = tokio::spawn(async move {
        let mut ticks = 0;
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,                     // asked to stop
                _ = sleep(Duration::from_millis(100)) => ticks += 1,
            }
        }
        // ...flush buffers, close connections, etc.
        ticks
    });
    sleep(run_for).await;
    let _ = stop_tx.send(());
    worker.await.expect("worker didn't panic")
}

/// Task 2: JoinSet -- a group of tasks; take results as they finish (not in
/// spawn order). Dropping the set aborts whatever is still running.
pub async fn joinset_sum(n: u64) -> u64 {
    let mut set = JoinSet::new();
    for i in 1..=n {
        set.spawn(compute(i));
    }
    let mut total = 0;
    while let Some(res) = set.join_next().await {
        total += res.expect("task panicked");
    }
    total
}

/// Task 3: at most `limit` tasks run the guarded section at once. Returns the
/// highest concurrency actually observed.
pub async fn limited_concurrency(tasks: usize, limit: usize) -> usize {
    let semaphore = Arc::new(Semaphore::new(limit));
    let running = Arc::new(AtomicUsize::new(0));
    let max_seen = Arc::new(AtomicUsize::new(0));
    let mut set = JoinSet::new();
    for _ in 0..tasks {
        let (semaphore, running, max_seen) = (semaphore.clone(), running.clone(), max_seen.clone());
        set.spawn(async move {
            // The permit is released when `_permit` is dropped.
            let _permit = semaphore
                .acquire_owned()
                .await
                .expect("semaphore never closed");
            let now = running.fetch_add(1, Ordering::SeqCst) + 1;
            max_seen.fetch_max(now, Ordering::SeqCst);
            sleep(Duration::from_millis(50)).await;
            running.fetch_sub(1, Ordering::SeqCst);
        });
    }
    while set.join_next().await.is_some() {}
    max_seen.load(Ordering::SeqCst)
}

/// Real CPU-bound work must not run on the async worker threads -- it would
/// starve every other task. `spawn_blocking` moves it to a separate pool.
pub async fn cpu_heavy_sum(n: u64) -> u64 {
    tokio::task::spawn_blocking(move || (0..=n).sum())
        .await
        .expect("blocking task")
}

pub async fn run() {
    let results: Vec<u64> = spawn_ten()
        .await
        .into_iter()
        .map(|r| r.expect("task ok"))
        .collect();
    println!("spawned 10: {results:?}");
    println!("aborted task reports cancelled: {}", abort_a_task().await);
    println!(
        "cooperative worker ticked {} times in 350ms",
        cancel_cooperatively(Duration::from_millis(350)).await
    );
    println!("JoinSet sum of squares 1..=10 = {}", joinset_sum(10).await);
    println!(
        "20 tasks, semaphore of 3: max concurrent = {}",
        limited_concurrency(20, 3).await
    );
    println!(
        "spawn_blocking sum 0..=10_000_000 = {}",
        cpu_heavy_sum(10_000_000).await
    );
}
