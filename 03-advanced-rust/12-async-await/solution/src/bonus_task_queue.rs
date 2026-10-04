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
        Job {
            id,
            priority,
            data: data.to_string(),
            fail_times: 0,
        }
    }

    pub fn failing(mut self, times: u32) -> Self {
        self.fail_times = times;
        self
    }
}

// ---- The minimal fix of the exercise's code ---------------------------------

/// Workers share one receiver. The lock is held only while waiting for the
/// next job, so a slow job doesn't block the others from picking up work.
/// `tokio::sync::Mutex` (not std's) because the guard lives across `.await`.
pub async fn shared_receiver_pool(num_workers: usize, jobs: Vec<Job>) -> Vec<(usize, u64)> {
    let (tx, rx) = mpsc::channel::<Job>(100);
    let rx = Arc::new(tokio::sync::Mutex::new(rx));
    let (done_tx, mut done_rx) = mpsc::unbounded_channel();

    let mut handles = Vec::new();
    for worker_id in 0..num_workers {
        let rx = Arc::clone(&rx);
        let done_tx = done_tx.clone();
        handles.push(tokio::spawn(async move {
            loop {
                let job = rx.lock().await.recv().await; // guard dropped at end of statement
                let Some(job) = job else { break }; // None: channel closed and empty
                sleep(Duration::from_millis(100)).await;
                let _ = done_tx.send((worker_id, job.id));
            }
        }));
    }
    drop(done_tx);

    for job in jobs {
        tx.send(job).await.expect("workers are running");
    }
    drop(tx); // signal completion
    for h in handles {
        h.await.expect("worker panicked");
    }
    let mut done = Vec::new();
    while let Some(d) = done_rx.recv().await {
        done.push(d);
    }
    done
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
        QueueConfig {
            workers: 4,
            max_attempts: 3,
            jobs_per_second: None,
            work_time: Duration::from_millis(100),
        }
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
        self.job
            .priority
            .cmp(&other.job.priority)
            .then_with(|| other.job.id.cmp(&self.job.id))
    }
}

impl PartialOrd for Queued {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
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
    let limiter = config.jobs_per_second.map(|n| {
        let mut ticker = interval(Duration::from_secs(1) / n.max(1));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        tokio::sync::Mutex::new(ticker)
    });
    let shared = Arc::new(Shared {
        pending: AtomicUsize::new(jobs.len()),
        heap: Mutex::new(
            jobs.into_iter()
                .map(|job| Queued { job, attempt: 1 })
                .collect(),
        ),
        notify: Notify::new(),
        limiter,
        config,
    });

    let (results_tx, mut results_rx) = mpsc::unbounded_channel();
    let workers: Vec<_> = (0..shared.config.workers.max(1))
        .map(|id| tokio::spawn(worker(id, Arc::clone(&shared), results_tx.clone())))
        .collect();
    drop(results_tx);

    for w in workers {
        w.await.expect("worker panicked");
    }
    let mut results = Vec::new();
    while let Some(r) = results_rx.recv().await {
        results.push(r);
    }
    results
}

async fn worker(id: usize, shared: Arc<Shared>, results: mpsc::UnboundedSender<JobResult>) {
    loop {
        // Register interest *before* checking, so a notify_waiters() that
        // happens between the check and the wait isn't lost.
        let notified = shared.notify.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();

        if shared.pending.load(Ordering::SeqCst) == 0 {
            return; // everything is finished
        }
        let next = shared.heap.lock().expect("heap lock").pop();
        let Some(Queued { job, attempt }) = next else {
            // Heap is empty but jobs are still in flight elsewhere (they may
            // be re-queued). Wait for news.
            notified.await;
            continue;
        };

        if let Some(limiter) = &shared.limiter {
            limiter.lock().await.tick().await;
        }
        sleep(shared.config.work_time).await;

        let failed = attempt <= job.fail_times;
        if failed && attempt < shared.config.max_attempts {
            shared.heap.lock().expect("heap lock").push(Queued {
                job,
                attempt: attempt + 1,
            });
        } else {
            let outcome = if failed {
                Outcome::Failed { attempts: attempt }
            } else {
                Outcome::Succeeded {
                    attempts: attempt,
                    output: job.data.to_uppercase(),
                }
            };
            let _ = results.send(JobResult {
                id: job.id,
                worker: id,
                outcome,
            });
            shared.pending.fetch_sub(1, Ordering::SeqCst);
        }
        shared.notify.notify_waiters();
    }
}

pub async fn run() {
    println!("-- the exercise's pool, fixed with a shared receiver --");
    let jobs = (0..8)
        .map(|i| Job::new(i, 0, &format!("Job {i}")))
        .collect();
    let mut done = shared_receiver_pool(4, jobs).await;
    done.sort();
    println!("(worker, job) pairs: {done:?}");

    println!("\n-- priority, retries, results, rate limit --");
    let jobs = vec![
        Job::new(1, 1, "low"),
        Job::new(2, 9, "urgent"),
        Job::new(3, 5, "flaky").failing(2),
        Job::new(4, 5, "doomed").failing(10),
        Job::new(5, 9, "also urgent"),
    ];
    let config = QueueConfig {
        workers: 1,
        jobs_per_second: Some(20),
        ..QueueConfig::default()
    };
    let start = tokio::time::Instant::now();
    for r in run_queue(jobs, config).await {
        println!("job {} on worker {}: {:?}", r.id, r.worker, r.outcome);
    }
    println!(
        "took {:.2?} (1 worker, so completion order = priority order)",
        start.elapsed()
    );
}
