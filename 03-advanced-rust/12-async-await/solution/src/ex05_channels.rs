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
    let (tx, mut rx) = mpsc::channel(32);
    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.expect("receiver is alive");
        }
    });
    let mut got = Vec::new();
    while let Some(msg) = rx.recv().await {
        got.push(msg);
    }
    got
}

/// Task 1: many producers. Each gets a clone of `tx`; the original must be
/// dropped too, or `recv()` waits forever for a sender that will never send.
pub async fn multiple_producers(producers: u32, per_producer: u32) -> Vec<(u32, u32)> {
    let (tx, mut rx) = mpsc::channel(8); // small buffer: senders wait when it's full (backpressure)
    for p in 0..producers {
        let tx = tx.clone();
        tokio::spawn(async move {
            for i in 0..per_producer {
                tx.send((p, i)).await.expect("receiver is alive");
            }
        });
    }
    drop(tx); // <- forget this and the loop below never ends
    let mut got = Vec::new();
    while let Some(msg) = rx.recv().await {
        got.push(msg);
    }
    got.sort();
    got
}

/// Task 2: broadcast. Every subscriber receives every message sent *after* it
/// subscribed. A subscriber that falls more than `capacity` messages behind
/// gets `RecvError::Lagged` and skips ahead.
pub async fn broadcast_to(subscribers: usize, messages: &[&'static str]) -> Vec<Vec<&'static str>> {
    let (tx, _) = broadcast::channel(16);
    let handles: Vec<_> = (0..subscribers)
        .map(|_| {
            let mut rx = tx.subscribe();
            tokio::spawn(async move {
                let mut got = Vec::new();
                while let Ok(msg) = rx.recv().await {
                    got.push(msg);
                }
                got
            })
        })
        .collect();
    for m in messages {
        tx.send(*m).expect("there are subscribers");
    }
    drop(tx); // closes the channel: each rx.recv() returns Err(Closed)
    let mut all = Vec::new();
    for h in handles {
        all.push(h.await.expect("subscriber task"));
    }
    all
}

/// Task 3: oneshot -- the reply half of request/response. We send a request
/// carrying a oneshot::Sender; the worker answers on it exactly once.
pub struct Request {
    pub n: u64,
    pub reply: oneshot::Sender<u64>,
}

pub fn spawn_squarer() -> mpsc::Sender<Request> {
    let (tx, mut rx) = mpsc::channel::<Request>(8);
    tokio::spawn(async move {
        while let Some(req) = rx.recv().await {
            // The asker may have given up (dropped its receiver); that's fine.
            let _ = req.reply.send(req.n * req.n);
        }
    });
    tx
}

pub async fn ask_square(worker: &mpsc::Sender<Request>, n: u64) -> Option<u64> {
    let (reply, answer) = oneshot::channel();
    worker.send(Request { n, reply }).await.ok()?;
    answer.await.ok()
}

pub async fn run() {
    println!("single producer: {:?}", single_producer().await);
    let got = multiple_producers(3, 4).await;
    println!("3 producers x 4 messages: {} received: {got:?}", got.len());
    println!(
        "broadcast to 3: {:?}",
        broadcast_to(3, &["a", "b", "c"]).await
    );
    let worker = spawn_squarer();
    println!(
        "oneshot request/response: 12^2 = {:?}",
        ask_square(&worker, 12).await
    );
}
