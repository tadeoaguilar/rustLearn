//! Exercise 6: events, delivered at least once.
//!
//! Services also talk *asynchronously*: Orders publishes "OrderPlaced" and
//! doesn't care who listens -- notifications, analytics, shipping. A broker
//! (RabbitMQ, Kafka, SQS, NATS) keeps the events until each consumer group
//! has processed them.
//!
//! Brokers deliver **at least once**: a consumer that crashes after doing
//! the work but before acknowledging gets the event again. So consumers must
//! be **idempotent** -- typically by remembering the ids they've processed.
//!
//! This is an in-memory broker with the same semantics: per-group queues,
//! acknowledgements, redelivery after a visibility timeout, and a dead-letter
//! queue for events that keep failing. Time is passed in.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub type EventId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery<E> {
    pub id: EventId,
    pub event: E,
    /// 1 on the first delivery.
    pub attempt: u32,
}

#[derive(Debug)]
struct Group<E> {
    ready: VecDeque<(EventId, E, u32)>,
    /// Delivered, not yet acknowledged: (event, attempt, redeliver at).
    in_flight: BTreeMap<EventId, (E, u32, Instant)>,
    dead: Vec<(EventId, E)>,
}

#[derive(Debug)]
struct Inner<E> {
    next_id: EventId,
    groups: BTreeMap<String, Group<E>>,
}

#[derive(Debug)]
pub struct Bus<E> {
    inner: Mutex<Inner<E>>,
    visibility_timeout: Duration,
    max_deliveries: u32,
}

impl<E: Clone> Bus<E> {
    pub fn new(visibility_timeout: Duration, max_deliveries: u32) -> Bus<E> {
        todo!("Exercise 6")
    }

    /// Creates a consumer group. It receives events published from now on.
    pub fn subscribe(&self, group: &str) {
        todo!("Exercise 6")
    }

    /// Every group gets its own copy.
    pub fn publish(&self, event: E) -> EventId {
        todo!("Exercise 6")
    }

    /// The next event for `group`, if any. Events whose visibility timeout
    /// expired go back to the queue first -- or to the dead letters once
    /// they've been delivered `max_deliveries` times.
    pub fn poll(&self, group: &str, now: Instant) -> Option<Delivery<E>> {
        todo!("Exercise 6")
    }

    /// Done: never deliver this event to this group again.
    pub fn ack(&self, group: &str, id: EventId) {
        todo!("Exercise 6")
    }

    /// Failed: make it available again right away (counts as a delivery).
    pub fn nack(&self, group: &str, id: EventId, now: Instant) {
        todo!("Exercise 6")
    }

    pub fn dead_letters(&self, group: &str) -> Vec<(EventId, E)> {
        todo!("Exercise 6")
    }

    /// Events waiting or in flight for `group`.
    pub fn pending(&self, group: &str) -> usize {
        todo!("Exercise 6")
    }
}

/// Wraps a handler so each event id is processed once, however often it's
/// delivered. (In production the "seen" set lives in the same database
/// transaction as the handler's effects.)
#[derive(Debug, Default)]
pub struct Idempotent {
    seen: HashSet<EventId>,
}

impl Idempotent {
    pub fn new() -> Idempotent {
        todo!("Exercise 6")
    }

    /// Runs `handler` unless `id` was already processed; true if it ran.
    pub fn handle(&mut self, id: EventId, handler: impl FnOnce()) -> bool {
        todo!("Exercise 6")
    }
}
