//! Exercise 7: a tiny load generator, to compare the two servers.
//!
//! `concurrency` tasks each send `requests / concurrency` requests back to
//! back and record every latency. Real tools (oha, wrk, k6) do the same with
//! more care; the point here is the *shape* of the numbers, and that you
//! measure instead of guessing.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LoadReport {
    pub requests: usize,
    pub errors: usize,
    pub elapsed: Duration,
    pub p50: Duration,
    pub p99: Duration,
}

impl LoadReport {
    pub fn requests_per_second(&self) -> f64 {
        todo!("Exercise 7")
    }
}

pub async fn load_test(url: &str, requests: usize, concurrency: usize) -> LoadReport {
    todo!("Exercise 7")
}
