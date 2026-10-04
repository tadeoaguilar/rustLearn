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
        FakeWeb {
            pages,
            latency,
            fetches: AtomicUsize::new(0),
        }
    }

    /// `n` pages "/0" .. "/n-1". Page i links to 2i+1 and 2i+2 (a binary
    /// tree), back to "/0", and to a page that doesn't exist.
    pub fn binary_tree(n: usize, latency: Duration) -> Self {
        let pages = (0..n)
            .map(|i| {
                let mut links: Vec<String> = [2 * i + 1, 2 * i + 2]
                    .iter()
                    .filter(|&&c| c < n)
                    .map(|c| format!("/{c}"))
                    .collect();
                links.push("/0".to_string());
                links.push("/missing".to_string());
                (format!("/{i}"), links)
            })
            .collect();
        FakeWeb::new(pages, latency)
    }

    /// Simulates a network request: sleeps, then returns the page's links.
    pub fn fetch(&self, url: &str) -> Option<Vec<String>> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        thread::sleep(self.latency);
        self.pages.get(url).cloned()
    }

    pub fn fetch_count(&self) -> usize {
        self.fetches.load(Ordering::SeqCst)
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
    let (tx, rx) = channel::unbounded::<(String, usize)>(); // crossbeam: multi-consumer
    let visited = Arc::new(Mutex::new(HashSet::from([start.to_string()])));
    let found = Arc::new(Mutex::new(BTreeSet::new()));
    let pending = Arc::new(AtomicUsize::new(1));
    tx.send((start.to_string(), 0)).expect("receiver alive");

    thread::scope(|s| {
        for _ in 0..workers.max(1) {
            let (tx, rx) = (tx.clone(), rx.clone());
            let (web, visited, found, pending) = (&web, &visited, &found, &pending);
            s.spawn(move || {
                loop {
                    let (url, depth) = match rx.recv_timeout(Duration::from_millis(2)) {
                        Ok(job) => job,
                        Err(_) if pending.load(Ordering::SeqCst) == 0 => break, // all done
                        Err(_) => continue, // others are still working; check again
                    };
                    if let Some(links) = web.fetch(&url) {
                        found.lock().unwrap().insert(url);
                        if depth < max_depth {
                            for link in links {
                                // insert() is false if already present: each URL is queued once.
                                if visited.lock().unwrap().insert(link.clone()) {
                                    pending.fetch_add(1, Ordering::SeqCst);
                                    tx.send((link, depth + 1)).expect("receiver alive");
                                }
                            }
                        }
                    }
                    pending.fetch_sub(1, Ordering::SeqCst);
                }
            });
        }
    });

    Arc::try_unwrap(found)
        .expect("workers are done")
        .into_inner()
        .unwrap()
}

pub fn run() {
    let web = Arc::new(FakeWeb::binary_tree(40, Duration::from_millis(10)));
    for workers in [1, 4, 8] {
        let before = web.fetch_count();
        let start = std::time::Instant::now();
        let pages = crawl(Arc::clone(&web), "/0", 10, workers);
        println!(
            "{workers} worker(s): {} pages, {} fetches, {:?}",
            pages.len(),
            web.fetch_count() - before,
            start.elapsed()
        );
    }
    let shallow = crawl(
        Arc::new(FakeWeb::binary_tree(40, Duration::ZERO)),
        "/0",
        2,
        4,
    );
    println!("depth 2: {shallow:?}");
}
