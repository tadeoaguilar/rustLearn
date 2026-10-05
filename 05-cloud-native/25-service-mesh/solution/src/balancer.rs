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
        Balancer {
            endpoints: urls
                .iter()
                .map(|u| Endpoint {
                    url: u.clone(),
                    in_flight: 0,
                    consecutive_failures: 0,
                    ejected_until: None,
                })
                .collect(),
            failure_threshold,
            ejection_time,
            max_ejected_fraction: 0.5,
        }
    }

    fn available(&self, now: Instant) -> Vec<usize> {
        (0..self.endpoints.len())
            .filter(|&i| self.endpoints[i].ejected_until.is_none_or(|t| t <= now))
            .collect()
    }

    pub fn is_ejected(&self, url: &str, now: Instant) -> bool {
        self.endpoints
            .iter()
            .any(|e| e.url == url && e.ejected_until.is_some_and(|t| t > now))
    }

    /// P2C between the endpoints at positions `a` and `b` (mod the number of
    /// available endpoints; the caller passes two random numbers). Marks the
    /// chosen one as having one more request in flight.
    pub fn pick(&mut self, now: Instant, a: usize, b: usize) -> Option<(usize, String)> {
        let available = self.available(now);
        if available.is_empty() {
            return None;
        }
        let (x, y) = (
            available[a % available.len()],
            available[b % available.len()],
        );
        let chosen = if self.endpoints[y].in_flight < self.endpoints[x].in_flight {
            y
        } else {
            x
        };
        self.endpoints[chosen].in_flight += 1;
        Some((chosen, self.endpoints[chosen].url.clone()))
    }

    /// Reports how a request to endpoint `index` went.
    pub fn finish(&mut self, index: usize, success: bool, now: Instant) {
        let ejected_now = self
            .endpoints
            .iter()
            .filter(|e| e.ejected_until.is_some_and(|t| t > now))
            .count();
        let max_ejected =
            (self.endpoints.len() as f64 * self.max_ejected_fraction).floor() as usize;
        let e = &mut self.endpoints[index];
        e.in_flight = e.in_flight.saturating_sub(1);
        if success {
            e.consecutive_failures = 0;
            return;
        }
        e.consecutive_failures += 1;
        if e.consecutive_failures >= self.failure_threshold
            && e.ejected_until.is_none_or(|t| t <= now)
            && ejected_now < max_ejected
        {
            e.ejected_until = Some(now + self.ejection_time);
            e.consecutive_failures = 0;
        }
    }

    pub fn in_flight(&self, url: &str) -> u32 {
        self.endpoints
            .iter()
            .find(|e| e.url == url)
            .map_or(0, |e| e.in_flight)
    }
}
