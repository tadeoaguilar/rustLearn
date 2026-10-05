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
        Bus {
            inner: Mutex::new(Inner {
                next_id: 1,
                groups: BTreeMap::new(),
            }),
            visibility_timeout,
            max_deliveries,
        }
    }

    /// Creates a consumer group. It receives events published from now on.
    pub fn subscribe(&self, group: &str) {
        self.inner
            .lock()
            .unwrap()
            .groups
            .entry(group.to_string())
            .or_insert_with(|| Group {
                ready: VecDeque::new(),
                in_flight: BTreeMap::new(),
                dead: Vec::new(),
            });
    }

    /// Every group gets its own copy.
    pub fn publish(&self, event: E) -> EventId {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        for group in inner.groups.values_mut() {
            group.ready.push_back((id, event.clone(), 0));
        }
        id
    }

    /// The next event for `group`, if any. Events whose visibility timeout
    /// expired go back to the queue first -- or to the dead letters once
    /// they've been delivered `max_deliveries` times.
    pub fn poll(&self, group: &str, now: Instant) -> Option<Delivery<E>> {
        let mut inner = self.inner.lock().unwrap();
        let g = inner.groups.get_mut(group)?;
        let expired: Vec<EventId> = g
            .in_flight
            .iter()
            .filter(|(_, (_, _, at))| *at <= now)
            .map(|(id, _)| *id)
            .collect();
        for id in expired {
            let (event, attempt, _) = g.in_flight.remove(&id).unwrap();
            if attempt >= self.max_deliveries {
                g.dead.push((id, event));
            } else {
                g.ready.push_back((id, event, attempt));
            }
        }
        let (id, event, attempt) = g.ready.pop_front()?;
        g.in_flight.insert(
            id,
            (event.clone(), attempt + 1, now + self.visibility_timeout),
        );
        Some(Delivery {
            id,
            event,
            attempt: attempt + 1,
        })
    }

    /// Done: never deliver this event to this group again.
    pub fn ack(&self, group: &str, id: EventId) {
        if let Some(g) = self.inner.lock().unwrap().groups.get_mut(group) {
            g.in_flight.remove(&id);
        }
    }

    /// Failed: make it available again right away (counts as a delivery).
    pub fn nack(&self, group: &str, id: EventId, now: Instant) {
        if let Some(g) = self.inner.lock().unwrap().groups.get_mut(group)
            && let Some(entry) = g.in_flight.get_mut(&id)
        {
            entry.2 = now;
        }
    }

    pub fn dead_letters(&self, group: &str) -> Vec<(EventId, E)> {
        self.inner
            .lock()
            .unwrap()
            .groups
            .get(group)
            .map(|g| g.dead.clone())
            .unwrap_or_default()
    }

    /// Events waiting or in flight for `group`.
    pub fn pending(&self, group: &str) -> usize {
        self.inner
            .lock()
            .unwrap()
            .groups
            .get(group)
            .map_or(0, |g| g.ready.len() + g.in_flight.len())
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
        Idempotent::default()
    }

    /// Runs `handler` unless `id` was already processed; true if it ran.
    pub fn handle(&mut self, id: EventId, handler: impl FnOnce()) -> bool {
        if self.seen.insert(id) {
            handler();
            true
        } else {
            false
        }
    }
}
