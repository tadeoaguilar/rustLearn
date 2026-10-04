//! Bonus Challenge: Async Task Queue.
//!
//! First, the exercise's code doesn't compile:
//!
//! ```text
//! let worker_rx = rx.clone();
//!                    ^^^^^ error[E0599]: no method named `clone` found for struct `tokio::sync::mpsc::Receiver`
//! ```
//!
//! mpsc is *single*-consumer. The smallest fix shares one receiver behind an
//! `Arc<tokio::sync::Mutex<_>>` -- see `shared_receiver_pool`. (A
//! multi-consumer channel crate such as `async-channel` is the other option.)
//!
//! Then the four tasks, in `run_queue`:
//! 1. priority      -- jobs wait in a `BinaryHeap`, highest priority first
//! 2. retry         -- a failed job goes back in the heap until `max_attempts`
//! 3. results       -- every final outcome is sent back on a channel
//! 4. rate limiting -- workers share one `Interval` that ticks N times a second

use std::cmp::Ordering as CmpOrdering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{Notify, mpsc};
use tokio::time::{Duration, Interval, MissedTickBehavior, interval, sleep};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: u64,
    pub priority: u8,
    pub data: String,
    /// Simulated flakiness: the first `fail_times` attempts fail.
    pub fail_times: u32,
}

impl Job {
    pub fn new(id: u64, priority: u8, data: &str) -> Self {
        todo!("Bonus")
    }

    pub fn failing(mut self, times: u32) -> Self {
        todo!("Bonus")
    }
}

// ---- The minimal fix of the exercise's code ---------------------------------

/// Workers share one receiver. The lock is held only while waiting for the
/// next job, so a slow job doesn't block the others from picking up work.
/// `tokio::sync::Mutex` (not std's) because the guard lives across `.await`.
pub async fn shared_receiver_pool(num_workers: usize, jobs: Vec<Job>) -> Vec<(usize, u64)> {
    todo!("Bonus")
}

// ---- The full queue ---------------------------------------------------------

#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub workers: usize,
    pub max_attempts: u32,
    /// None = unlimited.
    pub jobs_per_second: Option<u32>,
    pub work_time: Duration,
}

impl Default for QueueConfig {
    fn default() -> Self {
        todo!("Bonus")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Succeeded { attempts: u32, output: String },
    Failed { attempts: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobResult {
    pub id: u64,
    pub worker: usize,
    pub outcome: Outcome,
}

/// A job waiting in the heap, with its attempt number.
#[derive(Debug, PartialEq, Eq)]
struct Queued {
    job: Job,
    attempt: u32,
}

/// BinaryHeap is a max-heap: the "greatest" element pops first. Greatest =
/// highest priority; among equals, the *lowest* id (FIFO-ish), hence reversed.
impl Ord for Queued {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        todo!("Bonus")
    }
}

impl PartialOrd for Queued {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other)) // always like this when Ord exists
    }
}

struct Shared {
    heap: Mutex<BinaryHeap<Queued>>, // std Mutex: never held across an .await
    notify: Notify,
    /// Jobs not yet finished for good (succeeded or out of attempts).
    pending: AtomicUsize,
    limiter: Option<tokio::sync::Mutex<Interval>>,
    config: QueueConfig,
}

/// Runs every job to a final outcome and returns the results in completion order.
pub async fn run_queue(jobs: Vec<Job>, config: QueueConfig) -> Vec<JobResult> {
    todo!("Bonus")
}

async fn worker(id: usize, shared: Arc<Shared>, results: mpsc::UnboundedSender<JobResult>) {
    todo!("Bonus")
}

pub async fn run() {
    todo!("Bonus")
}
