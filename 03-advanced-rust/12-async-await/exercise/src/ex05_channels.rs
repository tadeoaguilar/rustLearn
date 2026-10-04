//! Exercise 5: Channels.
//!
//! | Channel      | Senders | Receivers | Each message goes to |
//! |--------------|---------|-----------|----------------------|
//! | `mpsc`       | many    | one       | the receiver         |
//! | `broadcast`  | many    | many      | *every* receiver     |
//! | `oneshot`    | one     | one       | the receiver, once   |
//! | `watch`      | many    | many      | receivers see the latest value only |

use tokio::sync::{broadcast, mpsc, oneshot};

/// The exercise's example: one producer task, receive until the channel closes.
/// `recv()` returns None once every Sender is dropped -- the spawned task
/// drops `tx` when it finishes, which ends the loop.
pub async fn single_producer() -> Vec<i32> {
    todo!("Exercise 5")
}

/// Task 1: many producers. Each gets a clone of `tx`; the original must be
/// dropped too, or `recv()` waits forever for a sender that will never send.
pub async fn multiple_producers(producers: u32, per_producer: u32) -> Vec<(u32, u32)> {
    todo!("Exercise 5")
}

/// Task 2: broadcast. Every subscriber receives every message sent *after* it
/// subscribed. A subscriber that falls more than `capacity` messages behind
/// gets `RecvError::Lagged` and skips ahead.
pub async fn broadcast_to(subscribers: usize, messages: &[&'static str]) -> Vec<Vec<&'static str>> {
    todo!("Exercise 5")
}

/// Task 3: oneshot -- the reply half of request/response. We send a request
/// carrying a oneshot::Sender; the worker answers on it exactly once.
pub struct Request {
    pub n: u64,
    pub reply: oneshot::Sender<u64>,
}

pub fn spawn_squarer() -> mpsc::Sender<Request> {
    todo!("Exercise 5")
}

pub async fn ask_square(worker: &mpsc::Sender<Request>, n: u64) -> Option<u64> {
    todo!("Exercise 5")
}

pub async fn run() {
    todo!("Exercise 5")
}
