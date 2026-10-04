//! Exercise 2: Message Passing with std::sync::mpsc.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// Fan-out / fan-in. `mpsc` is multi-producer, *single*-consumer, so the
/// workers share one job receiver behind a Mutex (the Rust Book's thread
/// pool does the same). Each worker sends its partial counts back on a
/// second channel; the main thread merges them.
pub fn word_count_pipeline(documents: Vec<String>, workers: usize) -> BTreeMap<String, usize> {
    let (job_tx, job_rx) = mpsc::channel::<String>();
    let job_rx = Arc::new(Mutex::new(job_rx));
    let (result_tx, result_rx) = mpsc::channel::<HashMap<String, usize>>();

    for _ in 0..workers.max(1) {
        let job_rx = Arc::clone(&job_rx);
        let result_tx = result_tx.clone();
        thread::spawn(move || {
            let mut local = HashMap::new();
            loop {
                // The guard is a temporary: the lock is released at the end of
                // this statement, before the (slow) counting starts.
                let job = job_rx.lock().expect("lock").recv();
                let Ok(doc) = job else { break }; // Err: every sender is gone
                for (w, n) in count_words(&doc) {
                    *local.entry(w).or_insert(0) += n;
                }
            }
            let _ = result_tx.send(local);
        });
    }
    drop(result_tx); // only the workers' clones remain: result_rx ends when they're done

    for doc in documents {
        job_tx.send(doc).expect("workers are alive");
    }
    drop(job_tx); // no more jobs: the workers' recv() returns Err and they finish

    let mut total = BTreeMap::new();
    for partial in result_rx {
        for (w, n) in partial {
            *total.entry(w).or_insert(0) += n;
        }
    }
    total
}

/// Returns the most items the producer ever got ahead of the consumer.
/// With `sync_channel(capacity)` that's at most capacity + 1: `capacity`
/// waiting in the channel, plus one the consumer has taken but not yet counted.
/// With `None` an unbounded `channel()` is used, and the producer races ahead.
pub fn producer_lead(items: usize, capacity: Option<usize>) -> usize {
    let sent = Arc::new(AtomicUsize::new(0));
    let received = Arc::new(AtomicUsize::new(0));
    let max_lead = Arc::new(AtomicUsize::new(0));

    let (tx, rx): (Box<dyn Fn(usize) + Send>, mpsc::Receiver<usize>) = match capacity {
        Some(cap) => {
            let (tx, rx) = mpsc::sync_channel(cap);
            (Box::new(move |v| tx.send(v).expect("consumer alive")), rx)
        }
        None => {
            let (tx, rx) = mpsc::channel();
            (Box::new(move |v| tx.send(v).expect("consumer alive")), rx)
        }
    };

    let consumer = {
        let received = Arc::clone(&received);
        thread::spawn(move || {
            for _ in rx {
                thread::sleep(Duration::from_micros(200)); // slow consumer
                received.fetch_add(1, Ordering::SeqCst);
            }
        })
    };

    for i in 0..items {
        tx(i); // blocks while a bounded channel is full
        let s = sent.fetch_add(1, Ordering::SeqCst) + 1;
        max_lead.fetch_max(s - received.load(Ordering::SeqCst), Ordering::SeqCst);
    }
    drop(tx);
    consumer.join().expect("consumer panicked");
    max_lead.load(Ordering::SeqCst)
}

pub fn sample_documents() -> Vec<String> {
    [
        "the quick brown fox",
        "jumps over the lazy dog",
        "The dog barks; the fox runs.",
        "a quick brown dog",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn run() {
    let counts = word_count_pipeline(sample_documents(), 3);
    let top: Vec<_> = counts.iter().filter(|(_, n)| **n > 1).collect();
    println!("words seen more than once: {top:?}");
    println!(
        "sync_channel(2): producer was at most {} ahead",
        producer_lead(200, Some(2))
    );
    println!(
        "sync_channel(0): producer was at most {} ahead (rendezvous)",
        producer_lead(200, Some(0))
    );
    println!(
        "channel():       producer was at most {} ahead (unbounded)",
        producer_lead(200, None)
    );
}
