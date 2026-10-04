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
        todo!("Exercise 5")
    }

    pub fn size(&self) -> usize {
        todo!("Exercise 5")
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        todo!("Exercise 5")
    }

    /// Runs `f` on the pool and returns a receiver for its result -- a
    /// minimal "future". If `f` panics, the receiver gets `Err(RecvError)`.
    pub fn spawn<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        todo!("Exercise 5")
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        todo!("Exercise 5")
    }
}

impl Drop for ThreadPool {
    /// Graceful shutdown: dropping the Sender lets each worker finish the
    /// jobs still queued, then see `Err` from `recv()` and exit. Then we
    /// join them all.
    fn drop(&mut self) {
        // TODO Exercise 5: a todo!() here could abort the test run, so this is empty.
    }
}

pub fn run() {
    todo!("Exercise 5")
}
