//! Exercise 2: Message Passing with std::sync::mpsc.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

fn count_words(text: &str) -> HashMap<String, usize> {
    todo!("Exercise 2")
}

/// Fan-out / fan-in. `mpsc` is multi-producer, *single*-consumer, so the
/// workers share one job receiver behind a Mutex (the Rust Book's thread
/// pool does the same). Each worker sends its partial counts back on a
/// second channel; the main thread merges them.
pub fn word_count_pipeline(documents: Vec<String>, workers: usize) -> BTreeMap<String, usize> {
    todo!("Exercise 2")
}

/// Returns the most items the producer ever got ahead of the consumer.
/// With `sync_channel(capacity)` that's at most capacity + 1: `capacity`
/// waiting in the channel, plus one the consumer has taken but not yet counted.
/// With `None` an unbounded `channel()` is used, and the producer races ahead.
pub fn producer_lead(items: usize, capacity: Option<usize>) -> usize {
    todo!("Exercise 2")
}

pub fn sample_documents() -> Vec<String> {
    todo!("Exercise 2")
}

pub fn run() {
    todo!("Exercise 2")
}
