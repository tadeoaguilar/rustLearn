//! Exercise 5: A Thread Pool (Rust Book chapter 21, improved).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    // Option so Drop can take() and drop it *before* joining the workers.
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    /// # Panics
    /// If `size` is 0.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "a thread pool needs at least one thread");
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn size(&self) -> usize {
        self.workers.len()
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender
            .as_ref()
            .expect("pool is running")
            .send(Box::new(f))
            .expect("workers are alive");
    }

    /// Runs `f` on the pool and returns a receiver for its result -- a
    /// minimal "future". If `f` panics, the receiver gets `Err(RecvError)`.
    pub fn spawn<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        self.execute(move || {
            let _ = tx.send(f());
        });
        rx
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::Builder::new()
            .name(format!("pool-worker-{id}"))
            .spawn(move || {
                loop {
                    // `let job = ...lock().recv();` ends the statement -- and
                    // releases the lock -- before the job runs. Writing
                    // `while let Ok(job) = receiver.lock().unwrap().recv()`
                    // would hold the lock for the whole loop body.
                    let message = receiver.lock().expect("receiver lock").recv();
                    match message {
                        Ok(job) => {
                            // A panicking job must not take its worker down.
                            if catch_unwind(AssertUnwindSafe(job)).is_err() {
                                eprintln!("[pool] a job on worker {id} panicked; worker continues");
                            }
                        }
                        Err(_) => break, // the pool dropped its Sender: shut down
                    }
                }
            })
            .expect("spawn worker thread");
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    /// Graceful shutdown: dropping the Sender lets each worker finish the
    /// jobs still queued, then see `Err` from `recv()` and exit. Then we
    /// join them all.
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(t) = worker.thread.take() {
                t.join()
                    .unwrap_or_else(|_| eprintln!("[pool] worker {} panicked", worker.id));
            }
        }
    }
}

pub fn run() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    let pool = ThreadPool::new(4);
    let start = Instant::now();
    let results: Vec<_> = (0..4)
        .map(|i| {
            pool.spawn(move || {
                thread::sleep(Duration::from_millis(100));
                i * i
            })
        })
        .collect();
    let squares: Vec<i32> = results.into_iter().map(|rx| rx.recv().unwrap()).collect();
    println!(
        "4 x 100ms jobs on 4 workers: {squares:?} in {:?}",
        start.elapsed()
    );

    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    pool.execute(|| panic!("boom"));
    let after_panic = pool.spawn(|| "the worker survived").recv().unwrap();
    std::panic::set_hook(previous);
    println!("after a panicking job: {after_panic}");

    let done = Arc::new(AtomicUsize::new(0));
    {
        let pool = ThreadPool::new(2);
        for _ in 0..10 {
            let done = Arc::clone(&done);
            pool.execute(move || {
                thread::sleep(Duration::from_millis(5));
                done.fetch_add(1, Ordering::SeqCst);
            });
        }
    } // Drop waits for all 10 queued jobs
    println!(
        "jobs finished before the pool was dropped: {}",
        done.load(Ordering::SeqCst)
    );
}
