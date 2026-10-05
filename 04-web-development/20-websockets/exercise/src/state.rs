//! Shared server state: chat rooms (Exercise 3), per-user notification
//! channels (4), the collaborative document (6), counters for the dashboard (7).
//!
//! The pattern throughout: one `tokio::sync::broadcast` channel per topic.
//! Every connection interested in the topic subscribes; publishing sends one
//! message that each subscriber receives. A subscriber that falls too far
//! behind gets `RecvError::Lagged` instead of slowing everyone down.

use crate::protocol::{ChatLine, ServerMessage};
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

pub const HISTORY: usize = 20;
const CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy)]
pub struct HeartbeatConfig {
    /// How often the server pings.
    pub interval: Duration,
    /// Close the connection if nothing at all (not even a pong) arrives for this long.
    pub timeout: Duration,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        todo!("Exercises 3-7")
    }
}

struct Room {
    tx: broadcast::Sender<ServerMessage>,
    history: VecDeque<ChatLine>,
    members: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub version: u64,
    pub text: String,
}

#[derive(Clone)]
pub struct AppState {
    rooms: Arc<Mutex<HashMap<String, Room>>>,
    inboxes: Arc<Mutex<HashMap<String, broadcast::Sender<ServerMessage>>>>,
    pub document: Arc<Mutex<Document>>,
    pub document_tx: broadcast::Sender<ServerMessage>,
    pub connections: Arc<AtomicUsize>,
    pub messages_total: Arc<AtomicU64>,
    pub started: Instant,
    pub heartbeat: HeartbeatConfig,
    /// Bonus: how many connections /ws/flaky has accepted.
    pub flaky_connections: Arc<AtomicU64>,
}

impl AppState {
    pub fn new(heartbeat: HeartbeatConfig) -> Self {
        todo!("Exercises 3-7")
    }

    // ---- Exercise 3: rooms ----

    /// Adds `name` to `room` (creating it), announces it, and returns a
    /// receiver for the room's messages, the members, and the recent history.
    /// The subscription is taken *before* the announcement, so the joiner
    /// sees its own `Joined`.
    pub fn join(
        &self,
        room: &str,
        name: &str,
    ) -> (
        broadcast::Receiver<ServerMessage>,
        Vec<String>,
        Vec<ChatLine>,
    ) {
        todo!("Exercises 3-7")
    }

    pub fn say(&self, room: &str, from: &str, text: &str) {
        todo!("Exercises 3-7")
    }

    /// Announces the departure; an empty room is removed.
    pub fn leave(&self, room: &str, name: &str) {
        todo!("Exercises 3-7")
    }

    pub fn room_members(&self, room: &str) -> Vec<String> {
        todo!("Exercises 3-7")
    }

    // ---- Exercise 4: notifications ----

    pub fn subscribe_inbox(&self, user: &str) -> broadcast::Receiver<ServerMessage> {
        todo!("Exercises 3-7")
    }

    /// Returns how many open connections received it (a user may have several tabs).
    pub fn notify(&self, user: &str, title: &str, body: &str) -> usize {
        todo!("Exercises 3-7")
    }

    // ---- Exercise 6: the shared document ----

    /// Optimistic concurrency: an edit based on the current version wins and
    /// is broadcast; an edit based on an older version is rejected with the
    /// current text, and the client must merge and retry.
    pub fn edit(
        &self,
        by: &str,
        base_version: u64,
        text: &str,
    ) -> Result<ServerMessage, ServerMessage> {
        todo!("Exercises 3-7")
    }

    // ---- Exercise 7: metrics ----

    pub fn metrics(&self) -> ServerMessage {
        todo!("Exercises 3-7")
    }
}

/// Counts open WebSocket connections; decrements when dropped, however the
/// connection ends.
pub struct ConnectionGuard(Arc<AtomicUsize>);

impl ConnectionGuard {
    pub fn new(state: &AppState) -> Self {
        todo!("Exercises 3-7")
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        // TODO TODO: a todo!() here could abort the test run, so this is empty.
    }
}
