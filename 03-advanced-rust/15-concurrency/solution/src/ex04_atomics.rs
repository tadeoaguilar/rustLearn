//! Exercise 4: Atomics and Memory Ordering.
//!
//! | Ordering  | Guarantees                                                     | Use for |
//! |-----------|----------------------------------------------------------------|---------|
//! | `Relaxed` | the operation itself is atomic; no ordering with other memory | counters, statistics |
//! | `Release` (store) + `Acquire` (load) | everything written before the Release is visible after the Acquire that reads it | locks, publishing data |
//! | `SeqCst`  | Acquire/Release + one global order of all SeqCst operations   | when in doubt; rarely *needed* |

use std::cell::UnsafeCell;
use std::hint;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const THREADS: usize = 8;
pub const PER_THREAD: usize = 125_000; // 8 x 125,000 = 1,000,000

pub fn count_with_mutex() -> (usize, Duration) {
    let start = Instant::now();
    let counter = Arc::new(Mutex::new(0usize));
    thread::scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|| {
                for _ in 0..PER_THREAD {
                    *counter.lock().unwrap() += 1;
                }
            });
        }
    });
    let n = *counter.lock().unwrap();
    (n, start.elapsed())
}

/// `Relaxed` is enough: nobody reads the count until every thread is joined,
/// and `join` itself synchronises.
pub fn count_with_atomic() -> (usize, Duration) {
    let start = Instant::now();
    let counter = AtomicUsize::new(0);
    thread::scope(|s| {
        for _ in 0..THREADS {
            s.spawn(|| {
                for _ in 0..PER_THREAD {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    (counter.into_inner(), start.elapsed())
}

/// The fastest by far: no sharing at all until the end.
pub fn count_with_local_sums() -> (usize, Duration) {
    let start = Instant::now();
    let total = thread::scope(|s| {
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                s.spawn(|| {
                    let mut local = 0usize;
                    for _ in 0..PER_THREAD {
                        local = hint::black_box(local + 1);
                    }
                    local
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    });
    (total, start.elapsed())
}

/// A compare-and-swap loop: read, compute, try to install; if another thread
/// changed the value meanwhile, `compare_exchange_weak` fails and hands back
/// the new current value to retry with. (`fetch_max` does this for you.)
pub fn atomic_max(target: &AtomicU64, value: u64) -> u64 {
    let mut current = target.load(Ordering::Relaxed);
    while value > current {
        match target.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return value,
            Err(actual) => current = actual,
        }
    }
    current
}

/// A spin lock: a bool "locked" flag guarding a value in an UnsafeCell.
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

// SAFETY: the lock guarantees only one thread at a time gets a &mut T, so
// sharing &SpinLock across threads is fine as long as T may be *sent*.
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Acquire: everything the previous holder wrote before its Release
    /// unlock is visible to us. With Relaxed, we could take the lock and
    /// still see a stale value -- the lock would protect nothing.
    pub fn lock(&self) -> SpinGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            hint::spin_loop(); // tell the CPU we're busy-waiting
        }
        SpinGuard { lock: self }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

pub struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: holding the guard means we hold the lock: no other access.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: as above, and &mut self means this guard isn't shared.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for SpinGuard<'_, T> {
    /// Release: publishes our writes to the next thread that Acquires.
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

pub fn run() {
    for (name, (n, t)) in [
        ("Mutex<usize>", count_with_mutex()),
        ("AtomicUsize", count_with_atomic()),
        ("local sums", count_with_local_sums()),
    ] {
        println!("{name:<13} counted to {n} in {t:?}");
    }
    let max = AtomicU64::new(0);
    thread::scope(|s| {
        for v in [17, 3, 99, 42] {
            let max = &max;
            s.spawn(move || atomic_max(max, v));
        }
    });
    println!(
        "atomic_max of [17, 3, 99, 42] = {}",
        max.load(Ordering::Relaxed)
    );

    let lock = SpinLock::new(Vec::new());
    thread::scope(|s| {
        for t in 0..4 {
            let lock = &lock;
            s.spawn(move || {
                for i in 0..1_000 {
                    lock.lock().push(t * 1_000 + i);
                }
            });
        }
    });
    println!(
        "SpinLock<Vec>: {} items pushed from 4 threads",
        lock.into_inner().len()
    );
}
