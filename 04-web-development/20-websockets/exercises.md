# Exercises: WebSockets

One Axum server, several real-time features: echo, chat rooms, push
notifications, heartbeats, a collaborative document, a live dashboard — and a
client that reconnects when the connection drops.

**Setup**: `axum` (with the `ws` feature), `tokio`, `tokio-tungstenite` (the
client), `futures-util` and `serde` are in `exercise/Cargo.toml`.
`client.rs`'s `WsClient` is provided; use it to try your server.

---

## Exercise 1: Echo

**Difficulty**: Easy
**Time**: 20 minutes

`GET /ws/echo` upgrades to a WebSocket and sends back every text and binary
message unchanged.

```rust
async fn echo(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|socket| echo_loop(socket))
}
```

**Questions**: What does the HTTP exchange look like before the first message?
Who answers pings?

---

## Exercise 2: A Typed Protocol

**Difficulty**: Medium
**Time**: 30 minutes

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage { Join { room: String }, Say { text: String }, Leave, Ping, Edit { base_version: u64, text: String } }

pub enum ServerMessage { Welcome{..}, Joined{..}, History{..}, Message{..}, Left{..}, Error{..}, Pong,
                         Notification{..}, Document{..}, Updated{..}, Conflict{..}, Metrics{..} }
```

On the wire: `{"type": "join", "room": "rust"}`. A malformed message or a
binary frame gets a `{"type": "error", ...}` reply — the connection stays open.

---

## Exercise 3: Chat Rooms

**Difficulty**: Hard
**Time**: 75 minutes

`GET /ws/chat?name=alice` (no name → **400 before upgrading**). Then:
- on connect: `Welcome`
- `Join` → `History` (the last 20 lines) to the joiner, `Joined {members}` to everyone in the room
- `Say` → `Message` to everyone in the room, sender included; must have joined; 1–1000 characters
- `Join` another room leaves the current one; `Leave`; disconnecting also leaves → `Left`

Use one `tokio::sync::broadcast` channel per room. Each connection runs a
`tokio::select!` loop over "next frame from the client" and "next message from
my room".

**Hint**: a room you haven't joined yet is `Option<broadcast::Receiver<_>>`;
make a helper future that waits forever on `None`, so the `select!` branch can
always be there.

---

## Exercise 4: Push Notifications From HTTP

**Difficulty**: Medium
**Time**: 40 minutes

- `GET /ws/notifications?user=bob` subscribes to Bob's inbox
- `POST /notify/{user}` with `{"title": "...", "body": "..."}` → **202**
  `{"delivered": <open connections that got it>}`

Bob in two tabs gets it twice; Carol gets nothing.

---

## Exercise 5: Heartbeats

**Difficulty**: Medium
**Time**: 40 minutes

A client whose laptop lid closed doesn't send a TCP close — the server sees
silence. In the chat endpoint:
- send a `Ping` frame every `heartbeat.interval`
- remember when the client last sent *anything* (pongs included)
- if that's longer ago than `heartbeat.timeout`, close the connection (and leave the room)

`HeartbeatConfig` is a parameter so tests can use 50 ms / 200 ms instead of
15 s / 45 s.

---

## Exercise 6: A Collaborative Document

**Difficulty**: Hard
**Time**: 60 minutes

`GET /ws/doc?name=..`:
- on connect: `Document {version, text}`
- `Edit {base_version, text}`: if `base_version` is the current version, the
  edit wins: version + 1, `Updated {version, text, by}` to **every** editor;
  otherwise `Conflict {your_base, current_version, text}` to the sender only

This is optimistic concurrency — the same idea as ETags in module 17.

**Question**: real editors (Google Docs, Figma) merge concurrent edits instead
of rejecting them. What techniques do they use?

---

## Exercise 7: A Live Dashboard

**Difficulty**: Easy
**Time**: 25 minutes

`GET /ws/dashboard?every_ms=1000` pushes `Metrics {connections,
messages_total, uptime_ms}` on a timer. `connections` counts open WebSockets —
use a guard type whose `Drop` decrements the counter, so every exit path is
covered. Skip missed ticks if the client is slow.

---

## Bonus Challenge: Reconnecting With Backoff

**Difficulty**: Medium
**Time**: 40 minutes

`GET /ws/flaky` (provided) sends `"hello #n"` and hangs up. Write:

```rust
pub struct Backoff { pub base: Duration, pub max: Duration }
impl Backoff { pub fn delay(&self, attempt: u32) -> Duration; }   // base * 2^attempt, capped
pub async fn collect_with_reconnects(url: &str, connections: usize, backoff: Backoff) -> Vec<String>;
```

→ `["hello #1", "hello #2", "hello #3"]`. Why do real clients add *random
jitter* to the delay?

---

## Check Your Understanding

- [ ] Explain the HTTP upgrade handshake
- [ ] Design a typed message protocol
- [ ] Fan out messages with `broadcast` channels
- [ ] Handle every way a connection can end (close frame, error, timeout)
- [ ] Detect dead connections with heartbeats
- [ ] Push to WebSocket clients from ordinary HTTP handlers
- [ ] Reconnect with exponential backoff

---

## Additional Resources

- [RFC 6455: The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455)
- [axum WebSocket docs](https://docs.rs/axum/latest/axum/extract/ws/) and [chat example](https://github.com/tokio-rs/axum/tree/main/examples/chat)
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/)
- [websocat](https://github.com/vi/websocat) — curl for WebSockets
