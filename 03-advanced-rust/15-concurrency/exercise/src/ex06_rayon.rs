//! Exercise 6: Data Parallelism with Rayon.
//!
//! `par_iter()` splits the work across a global thread pool (one thread per
//! core) using work-stealing. Same adaptors as normal iterators; the closure
//! must be `Fn + Sync + Send` because it runs on many threads at once.

use rayon::prelude::*;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn sum_of_squares_seq(v: &[u64]) -> u64 {
    todo!("Exercise 6")
}

pub fn sum_of_squares_par(v: &[u64]) -> u64 {
    todo!("Exercise 6")
}

fn is_prime(n: u64) -> bool {
    todo!("Exercise 6")
}

pub fn count_primes_seq(limit: u64) -> usize {
    todo!("Exercise 6")
}

pub fn count_primes_par(limit: u64) -> usize {
    todo!("Exercise 6")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileStats {
    pub name: String,
    pub lines: usize,
    pub words: usize,
    pub bytes: usize,
}

fn stats_for(path: &Path) -> io::Result<FileStats> {
    todo!("Exercise 6")
}

/// Every `.txt` file in `dir`, processed in parallel. Per-file stats sorted by
/// name, plus the total. One unreadable file fails the whole call.
pub fn process_dir(dir: &Path) -> io::Result<(Vec<FileStats>, FileStats)> {
    todo!("Exercise 6")
}

pub fn time<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    todo!("Exercise 6")
}

pub fn run() {
    todo!("Exercise 6")
}
