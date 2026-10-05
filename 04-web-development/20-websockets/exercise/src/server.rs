//! Exercises 1 and 3-7: the WebSocket endpoints.
//!
//!   GET  /ws/echo                     Exercise 1
//!   GET  /ws/chat?name=alice          Exercises 2, 3, 5 (protocol, rooms, heartbeat)
//!   GET  /ws/notifications?user=bob   Exercise 4 ...
//!   POST /notify/{user}               ... pushed from a normal HTTP request
//!   GET  /ws/doc?name=alice           Exercise 6
//!   GET  /ws/dashboard?every_ms=1000  Exercise 7
//!   GET  /ws/flaky                    Bonus: says hello, then hangs up
//!
//! Each connection is one task running a `select!` loop over everything it
//! waits for: the client's next frame, broadcast messages for it, timers.

use crate::protocol::{ClientMessage, ServerMessage, parse_client};
use crate::state::{AppState, ConnectionGuard};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

/// Sends one protocol message. `false` means the client is gone.
async fn send(socket: &mut WebSocket, msg: &ServerMessage) -> bool {
    todo!("Exercises 1-7")
}

/// Waits on an optional subscription: forever if there isn't one. Lets a
/// `select!` branch exist even before the client has joined a room.
async fn recv_opt(
    rx: &mut Option<broadcast::Receiver<ServerMessage>>,
) -> Result<ServerMessage, RecvError> {
    todo!("Exercises 1-7")
}

// ---- Exercise 1: echo ---------------------------------------------------------------

/// The upgrade handshake is an ordinary HTTP request; `WebSocketUpgrade`
/// answers it with 101 Switching Protocols, then `on_upgrade` gets the socket.
async fn echo(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    todo!("Exercises 1-7")
}

async fn echo_loop(mut socket: WebSocket, state: AppState) {
    todo!("Exercises 1-7")
}

// ---- Exercises 2, 3, 5: chat with rooms and heartbeats --------------------------------

#[derive(Deserialize)]
struct NameQuery {
    name: Option<String>,
}

async fn chat(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<NameQuery>,
) -> Response {
    todo!("Exercises 1-7")
}

async fn chat_loop(mut socket: WebSocket, state: AppState, name: String) {
    todo!("Exercises 1-7")
}

// ---- Exercise 4: notifications pushed from HTTP ---------------------------------------

#[derive(Deserialize)]
struct UserQuery {
    user: String,
}

async fn notifications(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<UserQuery>,
) -> Response {
    todo!("Exercises 1-7")
}

async fn notifications_loop(mut socket: WebSocket, state: AppState, user: String) {
    todo!("Exercises 1-7")
}

#[derive(Deserialize)]
struct NotifyBody {
    title: String,
    body: String,
}

/// 202 Accepted: handed to the open connections; delivery isn't confirmed.
async fn notify(
    State(state): State<AppState>,
    Path(user): Path<String>,
    Json(b): Json<NotifyBody>,
) -> Response {
    todo!("Exercises 1-7")
}

// ---- Exercise 6: collaborative document --------------------------------------------------

async fn doc(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<NameQuery>,
) -> Response {
    todo!("Exercises 1-7")
}

async fn doc_loop(mut socket: WebSocket, state: AppState, name: String) {
    todo!("Exercises 1-7")
}

// ---- Exercise 7: live dashboard ----------------------------------------------------------

#[derive(Deserialize)]
struct DashboardQuery {
    every_ms: Option<u64>,
}

async fn dashboard(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<DashboardQuery>,
) -> Response {
    todo!("Exercises 1-7")
}

async fn dashboard_loop(mut socket: WebSocket, state: AppState, every: Duration) {
    todo!("Exercises 1-7")
}

// ---- Bonus: a flaky endpoint to reconnect to ---------------------------------------------

async fn flaky(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    todo!("Exercises 1-7")
}

async fn health() -> &'static str {
    todo!("Exercises 1-7")
}

pub fn router(state: AppState) -> Router {
    todo!("Exercises 1-7")
}
