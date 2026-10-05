//! Exercise 2: a typed message protocol.
//!
//! WebSockets carry bytes or text; the *meaning* is up to you. A tagged enum
//! per direction turns every message into one JSON object with a `type`:
//!
//! ```json
//! {"type": "join", "room": "rust"}
//! {"type": "message", "room": "rust", "from": "alice", "text": "hi"}
//! ```
//!
//! serde does the parsing; a malformed message becomes a `ServerMessage::Error`
//! instead of a crash or a silently dropped frame.

use serde::{Deserialize, Serialize};

/// Client -> server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Join { room: String },
    Say { text: String },
    Leave,
    Ping,
    Edit { base_version: u64, text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatLine {
    pub from: String,
    pub text: String,
}

/// Server -> client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        name: String,
    },
    Joined {
        room: String,
        name: String,
        members: Vec<String>,
    },
    History {
        room: String,
        lines: Vec<ChatLine>,
    },
    Message {
        room: String,
        from: String,
        text: String,
    },
    Left {
        room: String,
        name: String,
    },
    Error {
        message: String,
    },
    Pong,
    Notification {
        title: String,
        body: String,
    },
    Document {
        version: u64,
        text: String,
    },
    Updated {
        version: u64,
        text: String,
        by: String,
    },
    Conflict {
        your_base: u64,
        current_version: u64,
        text: String,
    },
    Metrics {
        connections: usize,
        messages_total: u64,
        uptime_ms: u64,
    },
}

impl ServerMessage {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("server messages always serialise")
    }
}

pub fn parse_client(text: &str) -> Result<ClientMessage, ServerMessage> {
    serde_json::from_str(text).map_err(|e| ServerMessage::Error {
        message: format!("bad message: {e}"),
    })
}
