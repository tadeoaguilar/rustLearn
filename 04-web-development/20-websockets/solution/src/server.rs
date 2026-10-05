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
    socket
        .send(Message::Text(msg.to_json().into()))
        .await
        .is_ok()
}

/// Waits on an optional subscription: forever if there isn't one. Lets a
/// `select!` branch exist even before the client has joined a room.
async fn recv_opt(
    rx: &mut Option<broadcast::Receiver<ServerMessage>>,
) -> Result<ServerMessage, RecvError> {
    match rx {
        Some(rx) => rx.recv().await,
        None => std::future::pending().await,
    }
}

// ---- Exercise 1: echo ---------------------------------------------------------------

/// The upgrade handshake is an ordinary HTTP request; `WebSocketUpgrade`
/// answers it with 101 Switching Protocols, then `on_upgrade` gets the socket.
async fn echo(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| echo_loop(socket, state))
}

async fn echo_loop(mut socket: WebSocket, state: AppState) {
    let _guard = ConnectionGuard::new(&state);
    // recv() returns None when the connection is gone. Pings from the client
    // are answered with pongs automatically.
    while let Some(Ok(msg)) = socket.recv().await {
        let reply = match msg {
            Message::Text(t) => Message::Text(t),
            Message::Binary(b) => Message::Binary(b),
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) => continue,
        };
        if socket.send(reply).await.is_err() {
            break;
        }
    }
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
    // Validate *before* upgrading: a plain 400 is clearer than an upgraded
    // socket that closes immediately.
    let Some(name) = q.name.filter(|n| !n.trim().is_empty() && n.len() <= 32) else {
        return (
            StatusCode::BAD_REQUEST,
            "?name= is required (1-32 characters)",
        )
            .into_response();
    };
    ws.on_upgrade(move |socket| chat_loop(socket, state, name))
}

async fn chat_loop(mut socket: WebSocket, state: AppState, name: String) {
    let _guard = ConnectionGuard::new(&state);
    let mut room: Option<String> = None;
    let mut rx: Option<broadcast::Receiver<ServerMessage>> = None;
    let mut heartbeat = tokio::time::interval(state.heartbeat.interval);
    heartbeat.reset(); // the first tick of an interval is immediate; skip it
    let mut last_seen = Instant::now();

    if !send(&mut socket, &ServerMessage::Welcome { name: name.clone() }).await {
        return;
    }

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(Ok(msg)) = incoming else { break };     // closed or errored
                last_seen = Instant::now();                       // any frame counts, pongs included
                let text = match msg {
                    Message::Text(t) => t,
                    Message::Close(_) => break,
                    Message::Binary(_) => {
                        if !send(&mut socket, &ServerMessage::Error { message: "binary frames aren't supported".into() }).await { break }
                        continue;
                    }
                    Message::Ping(_) | Message::Pong(_) => continue,
                };
                let reply = match parse_client(&text) {
                    Err(error) => Some(error),
                    Ok(ClientMessage::Ping) => Some(ServerMessage::Pong),
                    Ok(ClientMessage::Join { room: new_room }) => {
                        if let Some(old) = room.take() {
                            state.leave(&old, &name);
                        }
                        let (new_rx, _members, lines) = state.join(&new_room, &name);
                        rx = Some(new_rx);
                        room = Some(new_room.clone());
                        Some(ServerMessage::History { room: new_room, lines })
                    }
                    Ok(ClientMessage::Say { text }) => match &room {
                        None => Some(ServerMessage::Error { message: "join a room first".into() }),
                        Some(_) if text.trim().is_empty() || text.len() > 1000 => {
                            Some(ServerMessage::Error { message: "messages must be 1-1000 characters".into() })
                        }
                        Some(r) => {
                            state.say(r, &name, &text); // everyone in the room, including us, gets it via rx
                            None
                        }
                    },
                    Ok(ClientMessage::Leave) => {
                        if let Some(old) = room.take() {
                            state.leave(&old, &name);
                        }
                        rx = None;
                        None
                    }
                    Ok(ClientMessage::Edit { .. }) => Some(ServerMessage::Error { message: "edits go to /ws/doc".into() }),
                };
                if let Some(reply) = reply && !send(&mut socket, &reply).await {
                    break;
                }
            }
            broadcast = recv_opt(&mut rx) => {
                let msg = match broadcast {
                    Ok(msg) => msg,
                    // A slow client: tell it, and carry on from the newest messages.
                    Err(RecvError::Lagged(n)) => ServerMessage::Error { message: format!("you missed {n} messages") },
                    Err(RecvError::Closed) => { rx = None; continue }
                };
                if !send(&mut socket, &msg).await { break }
            }
            // Exercise 5: TCP can't tell "idle" from "gone" (a laptop closed,
            // a phone in a tunnel). Ping regularly; no traffic at all within
            // `timeout` means the client is dead -- free its resources.
            _ = heartbeat.tick() => {
                if last_seen.elapsed() > state.heartbeat.timeout {
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() { break }
            }
        }
    }

    // However the loop ended -- close, error, timeout -- leave the room.
    if let Some(r) = room {
        state.leave(&r, &name);
    }
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
    ws.on_upgrade(move |socket| notifications_loop(socket, state, q.user))
}

async fn notifications_loop(mut socket: WebSocket, state: AppState, user: String) {
    let _guard = ConnectionGuard::new(&state);
    let mut rx = state.subscribe_inbox(&user);
    loop {
        tokio::select! {
            // We don't expect messages from the client, but must still read,
            // or we'd never notice it closing.
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                Some(Ok(_)) => {}
            },
            note = rx.recv() => match note {
                Ok(msg) => if !send(&mut socket, &msg).await { break },
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            },
        }
    }
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
    let delivered = state.notify(&user, &b.title, &b.body);
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "delivered": delivered })),
    )
        .into_response()
}

// ---- Exercise 6: collaborative document --------------------------------------------------

async fn doc(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<NameQuery>,
) -> Response {
    let name = q.name.unwrap_or_else(|| "anonymous".into());
    ws.on_upgrade(move |socket| doc_loop(socket, state, name))
}

async fn doc_loop(mut socket: WebSocket, state: AppState, name: String) {
    let _guard = ConnectionGuard::new(&state);
    // Subscribe before reading the snapshot: an edit landing in between then
    // arrives as an update instead of being lost.
    let mut rx = state.document_tx.subscribe();
    let snapshot = {
        let d = state.document.lock().unwrap();
        ServerMessage::Document {
            version: d.version,
            text: d.text.clone(),
        }
    };
    if !send(&mut socket, &snapshot).await {
        return;
    }
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let text = match incoming {
                    Some(Ok(Message::Text(t))) => t,
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => continue,
                };
                match parse_client(&text) {
                    // Accepted edits reach everyone (us included) through rx.
                    Ok(ClientMessage::Edit { base_version, text }) => {
                        if let Err(conflict) = state.edit(&name, base_version, &text)
                            && !send(&mut socket, &conflict).await
                        {
                            break;
                        }
                    }
                    Ok(_) => { if !send(&mut socket, &ServerMessage::Error { message: "only edit messages here".into() }).await { break } }
                    Err(e) => { if !send(&mut socket, &e).await { break } }
                }
            }
            update = rx.recv() => match update {
                Ok(msg) => if !send(&mut socket, &msg).await { break },
                Err(RecvError::Lagged(_)) => {
                    // Too far behind to replay: resend the whole document.
                    let d = state.document.lock().unwrap().clone();
                    if !send(&mut socket, &ServerMessage::Document { version: d.version, text: d.text }).await { break }
                }
                Err(RecvError::Closed) => break,
            },
        }
    }
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
    let every = Duration::from_millis(q.every_ms.unwrap_or(1000).max(10));
    ws.on_upgrade(move |socket| dashboard_loop(socket, state, every))
}

async fn dashboard_loop(mut socket: WebSocket, state: AppState, every: Duration) {
    let _guard = ConnectionGuard::new(&state);
    let mut ticker = tokio::time::interval(every);
    // If the client is slow, skip missed ticks instead of bursting to catch up.
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => if !send(&mut socket, &state.metrics()).await { break },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                Some(Ok(_)) => {}
            },
        }
    }
}

// ---- Bonus: a flaky endpoint to reconnect to ---------------------------------------------

async fn flaky(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |mut socket| async move {
        let n = state
            .flaky_connections
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;
        let _ = socket
            .send(Message::Text(format!("hello #{n}").into()))
            .await;
        let _ = socket.send(Message::Close(None)).await;
    })
}

async fn health() -> &'static str {
    "ok"
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ws/echo", get(echo))
        .route("/ws/chat", get(chat))
        .route("/ws/notifications", get(notifications))
        .route("/notify/{user}", post(notify))
        .route("/ws/doc", get(doc))
        .route("/ws/dashboard", get(dashboard))
        .route("/ws/flaky", get(flaky))
        .with_state(state)
}
