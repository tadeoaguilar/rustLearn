//! Exercise 7: load balancing with outlier detection.
//!
//! **Power of two choices**: pick two endpoints at random, send to the one
//! with fewer requests in flight. Almost as good as checking every endpoint,
//! at the cost of two -- and it naturally avoids slow ones (their requests
//! pile up). **Outlier detection**: an endpoint returning consecutive errors
//! is *ejected* for a while; never eject more than half, or a cluster-wide
//! problem becomes a total outage.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Endpoint {
    url: String,
    in_flight: u32,
    consecutive_failures: u32,
    ejected_until: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct Balancer {
    endpoints: Vec<Endpoint>,
    failure_threshold: u32,
    ejection_time: Duration,
    max_ejected_fraction: f64,
}

impl Balancer {
    pub fn new(urls: &[String], failure_threshold: u32, ejection_time: Duration) -> Balancer {
        todo!("Exercise 7")
    }

    fn available(&self, now: Instant) -> Vec<usize> {
        todo!("Exercise 7")
    }

    pub fn is_ejected(&self, url: &str, now: Instant) -> bool {
        todo!("Exercise 7")
    }

    /// P2C between the endpoints at positions `a` and `b` (mod the number of
    /// available endpoints; the caller passes two random numbers). Marks the
    /// chosen one as having one more request in flight.
    pub fn pick(&mut self, now: Instant, a: usize, b: usize) -> Option<(usize, String)> {
        todo!("Exercise 7")
    }

    /// Reports how a request to endpoint `index` went.
    pub fn finish(&mut self, index: usize, success: bool, now: Instant) {
        todo!("Exercise 7")
    }

    pub fn in_flight(&self, url: &str) -> u32 {
        todo!("Exercise 7")
    }
}
