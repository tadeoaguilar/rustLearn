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
        self.requests as f64 / self.elapsed.as_secs_f64()
    }
}

pub async fn load_test(url: &str, requests: usize, concurrency: usize) -> LoadReport {
    let client = reqwest::Client::new(); // one client: connection pooling
    let concurrency = concurrency.clamp(1, requests.max(1));
    let per_task = requests / concurrency;
    let start = Instant::now();
    let tasks = (0..concurrency).map(|_| {
        let client = client.clone();
        let url = url.to_string();
        tokio::spawn(async move {
            let mut latencies = Vec::with_capacity(per_task);
            let mut errors = 0;
            for _ in 0..per_task {
                let t = Instant::now();
                match client.get(&url).send().await {
                    Ok(r) if r.status().is_success() => {
                        let _ = r.bytes().await;
                    }
                    _ => errors += 1,
                }
                latencies.push(t.elapsed());
            }
            (latencies, errors)
        })
    });
    let mut latencies = Vec::with_capacity(requests);
    let mut errors = 0;
    for result in futures::future::join_all(tasks).await {
        let (l, e) = result.expect("load task panicked");
        latencies.extend(l);
        errors += e;
    }
    let elapsed = start.elapsed();
    latencies.sort();
    let pct = |p: f64| {
        latencies
            .get(((latencies.len() as f64 * p) as usize).min(latencies.len().saturating_sub(1)))
            .copied()
            .unwrap_or_default()
    };
    LoadReport {
        requests: latencies.len(),
        errors,
        elapsed,
        p50: pct(0.50),
        p99: pct(0.99),
    }
}
