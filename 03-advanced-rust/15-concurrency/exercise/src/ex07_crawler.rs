//! Exercise 7: A Concurrent Web Crawler over a simulated web.

use crossbeam::channel;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// An in-memory "internet": page -> links, with a fixed fetch latency.
pub struct FakeWeb {
    pages: HashMap<String, Vec<String>>,
    latency: Duration,
    fetches: AtomicUsize,
}

impl FakeWeb {
    pub fn new(pages: HashMap<String, Vec<String>>, latency: Duration) -> Self {
        todo!("Exercise 7")
    }

    /// `n` pages "/0" .. "/n-1". Page i links to 2i+1 and 2i+2 (a binary
    /// tree), back to "/0", and to a page that doesn't exist.
    pub fn binary_tree(n: usize, latency: Duration) -> Self {
        todo!("Exercise 7")
    }

    /// Simulates a network request: sleeps, then returns the page's links.
    pub fn fetch(&self, url: &str) -> Option<Vec<String>> {
        todo!("Exercise 7")
    }

    pub fn fetch_count(&self) -> usize {
        todo!("Exercise 7")
    }
}

/// Crawls breadth-first-ish from `start`, following links up to `max_depth`
/// (start is depth 0). Returns the pages that exist and were reached.
///
/// The hard part is knowing when to stop: the queue can be momentarily empty
/// while a worker is still fetching a page that will add more links.
/// `pending` counts URLs queued *or* being processed. A worker increments it
/// for each new link *before* decrementing it for the page it finished, so it
/// can only reach zero when there is truly nothing left.
pub fn crawl(web: Arc<FakeWeb>, start: &str, max_depth: usize, workers: usize) -> BTreeSet<String> {
    todo!("Exercise 7")
}

pub fn run() {
    todo!("Exercise 7")
}
