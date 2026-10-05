# 20 · WebSockets

## Overview

HTTP is request/response: the client asks, the server answers, the exchange
is over. Real-time features need the server to speak first — a chat message,
a notification, a new metric. A WebSocket starts as an HTTP request with
`Upgrade: websocket`, the server answers `101 Switching Protocols`, and from
then on both sides can send messages whenever they like over the same TCP
connection.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Echo | The upgrade handshake; the receive loop; text vs binary frames |
| 2 | Protocol | Tagged serde enums; errors as messages, not disconnects |
| 3 | Chat rooms | One `broadcast` channel per room; `select!` per connection; cleanup on every exit |
| 4 | Notifications | Pushing to WebSocket clients from a plain HTTP handler |
| 5 | Heartbeats | Detecting connections that died without saying goodbye |
| 6 | Collaborative doc | Versioned edits, broadcast on success, conflict on stale writes |
| 7 | Dashboard | Timer-driven pushes; counting connections with a `Drop` guard |
| Bonus | Reconnecting client | Exponential backoff (and why jitter) |

## Key Concepts

### One task per connection, one `select!` per task

```rust
loop {
    tokio::select! {
        frame = socket.recv()   => { /* the client said something */ }
        msg   = room_rx.recv()  => { /* someone else did */ }
        _     = heartbeat.tick()=> { /* time to ping, or give up */ }
    }
}
// whichever way the loop ended: leave rooms, decrement counters
```

Each `select!` branch is something the connection is waiting for. The code
after the loop runs however the connection ended — close frame, network error,
heartbeat timeout — which is where cleanup belongs.

### `broadcast` for fan-out

A `tokio::sync::broadcast` channel delivers every message to every receiver.
One per chat room, one per user inbox, one for the document: publishing is a
single `send`, and each connection forwards what its receiver gets. A receiver
that falls more than the channel's capacity behind gets `Lagged(n)` and skips
ahead — one slow client never slows the rest.

### Dead connections are silent

When a phone loses signal, no FIN or close frame is sent; the server just
stops hearing anything. Without heartbeats, those connections — and their
room memberships and buffers — live forever. Ping on an interval, and drop
connections that haven't sent anything (pongs count) within a timeout.

### State lives on one server

All rooms, inboxes and documents here are in one process's memory. With
several server instances behind a load balancer, Alice and Bob may connect to
different ones — so the broadcast must go through shared infrastructure
(Redis pub/sub, NATS, Postgres `LISTEN/NOTIFY`), and load balancers need
sticky sessions or must be fine with any instance serving any socket.

## Common Pitfalls

1. **Not reading from the socket** in push-only endpoints — you never notice the client leaving
2. **Cleanup only on the happy path** — leaving rooms must happen on errors and timeouts too
3. **Holding a `std::sync::Mutex` across `.await`** — lock, copy, unlock, then send
4. **No message size or rate limits** — one client can flood a room
5. **Auth only at connect time** — a token that expires mid-session isn't re-checked
6. **Reconnecting in a tight loop** — every client hammers the server the moment it comes back

## Running This Module

```bash
cargo run  -p m20-websockets -- demo                      # your code
cargo test -p m20-websockets-tests --features mine        # test your code
cargo run  -p m20-websockets-solution -- demo             # clients exercise every endpoint
cargo run  -p m20-websockets-solution -- serve            # ws://127.0.0.1:3000
cargo test -p m20-websockets-tests                        # 14 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercise list in
[the phase README](../README.md) (a chat server, real-time notifications, a
collaborative editor, a live dashboard).

- **The collaborative editor uses whole-document versioning**, not operational
  transforms or CRDTs — it teaches the server-side mechanics without a
  research project. ANSWERS.md points to the real techniques.
- **`client::WsClient` is provided** in the exercise crate; the tests and your
  demos need it before you write the bonus.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

Phase 4 is complete. Continue with
[Phase 5 · Cloud Native](../../05-cloud-native/), or build the
[CRUD API project](../../10-crud-api-project/).
