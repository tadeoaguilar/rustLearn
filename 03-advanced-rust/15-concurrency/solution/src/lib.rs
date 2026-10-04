//! Module 15 -- Concurrency. Reference solution.
//!
//! The compiler prevents data races through two marker traits:
//!   * `Send` -- a value can be *moved* to another thread
//!   * `Sync` -- a `&value` can be *shared* between threads
//!
//! `thread::spawn` requires `Send + 'static`; `Arc<T>` is Send+Sync only if T
//! is; `Mutex<T>` makes a Send T into a Sync one. Get those wrong and it
//! doesn't compile. Deadlocks and logic races, however, still compile -- those
//! are what these exercises are about.

pub mod bonus_philosophers;
pub mod ex01_threads;
pub mod ex02_channels;
pub mod ex03_shared_state;
pub mod ex04_atomics;
pub mod ex05_thread_pool;
pub mod ex06_rayon;
pub mod ex07_crawler;
pub mod ex08_lock_free;
