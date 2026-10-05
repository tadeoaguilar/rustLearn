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
        HeartbeatConfig {
            interval: Duration::from_secs(15),
            timeout: Duration::from_secs(45),
        }
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
        AppState {
            rooms: Arc::default(),
            inboxes: Arc::default(),
            document: Arc::new(Mutex::new(Document {
                version: 0,
                text: String::new(),
            })),
            document_tx: broadcast::channel(CHANNEL_CAPACITY).0,
            connections: Arc::default(),
            messages_total: Arc::default(),
            started: Instant::now(),
            heartbeat,
            flaky_connections: Arc::default(),
        }
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
        let mut rooms = self.rooms.lock().unwrap();
        let r = rooms.entry(room.to_string()).or_insert_with(|| Room {
            tx: broadcast::channel(CHANNEL_CAPACITY).0,
            history: VecDeque::new(),
            members: BTreeSet::new(),
        });
        let rx = r.tx.subscribe();
        r.members.insert(name.to_string());
        let members: Vec<String> = r.members.iter().cloned().collect();
        let _ = r.tx.send(ServerMessage::Joined {
            room: room.into(),
            name: name.into(),
            members: members.clone(),
        });
        (rx, members, r.history.iter().cloned().collect())
    }

    pub fn say(&self, room: &str, from: &str, text: &str) {
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(r) = rooms.get_mut(room) {
            r.history.push_back(ChatLine {
                from: from.into(),
                text: text.into(),
            });
            if r.history.len() > HISTORY {
                r.history.pop_front();
            }
            let _ = r.tx.send(ServerMessage::Message {
                room: room.into(),
                from: from.into(),
                text: text.into(),
            });
            self.messages_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Announces the departure; an empty room is removed.
    pub fn leave(&self, room: &str, name: &str) {
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(r) = rooms.get_mut(room) {
            r.members.remove(name);
            let _ = r.tx.send(ServerMessage::Left {
                room: room.into(),
                name: name.into(),
            });
            if r.members.is_empty() {
                rooms.remove(room);
            }
        }
    }

    pub fn room_members(&self, room: &str) -> Vec<String> {
        self.rooms
            .lock()
            .unwrap()
            .get(room)
            .map(|r| r.members.iter().cloned().collect())
            .unwrap_or_default()
    }

    // ---- Exercise 4: notifications ----

    pub fn subscribe_inbox(&self, user: &str) -> broadcast::Receiver<ServerMessage> {
        let mut inboxes = self.inboxes.lock().unwrap();
        inboxes
            .entry(user.to_string())
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe()
    }

    /// Returns how many open connections received it (a user may have several tabs).
    pub fn notify(&self, user: &str, title: &str, body: &str) -> usize {
        let inboxes = self.inboxes.lock().unwrap();
        match inboxes.get(user) {
            Some(tx) => tx
                .send(ServerMessage::Notification {
                    title: title.into(),
                    body: body.into(),
                })
                .unwrap_or(0),
            None => 0,
        }
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
        let mut doc = self.document.lock().unwrap();
        if base_version != doc.version {
            return Err(ServerMessage::Conflict {
                your_base: base_version,
                current_version: doc.version,
                text: doc.text.clone(),
            });
        }
        doc.version += 1;
        doc.text = text.to_string();
        let update = ServerMessage::Updated {
            version: doc.version,
            text: doc.text.clone(),
            by: by.into(),
        };
        let _ = self.document_tx.send(update.clone());
        Ok(update)
    }

    // ---- Exercise 7: metrics ----

    pub fn metrics(&self) -> ServerMessage {
        ServerMessage::Metrics {
            connections: self.connections.load(Ordering::Relaxed),
            messages_total: self.messages_total.load(Ordering::Relaxed),
            uptime_ms: self.started.elapsed().as_millis() as u64,
        }
    }
}

/// Counts open WebSocket connections; decrements when dropped, however the
/// connection ends.
pub struct ConnectionGuard(Arc<AtomicUsize>);

impl ConnectionGuard {
    pub fn new(state: &AppState) -> Self {
        state.connections.fetch_add(1, Ordering::Relaxed);
        ConnectionGuard(state.connections.clone())
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}
