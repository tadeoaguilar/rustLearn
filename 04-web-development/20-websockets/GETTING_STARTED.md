# Getting Started with 20 · WebSockets

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m20-websockets-solution -- demo
```

## What Is Already Here

```
20-websockets/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/          # ← YOUR WORKSPACE (package m20-websockets)
│   ├── protocol.rs        #   Ex 2
│   ├── state.rs           #   rooms, inboxes, document, metrics (Ex 3, 4, 6, 7)
│   ├── server.rs          #   the endpoints (Ex 1, 3-7)
│   ├── client.rs          #   WsClient PROVIDED; Backoff + reconnecting = bonus
│   └── main.rs            #   runner: demo / serve
├── solution/              # ← REFERENCE (m20-websockets-solution) + ANSWERS.md
└── tests/                 # ← 14 tests (m20-websockets-tests)
```

## The Commands You Need

```bash
cargo run  -p m20-websockets -- demo
cargo test -p m20-websockets-tests --features mine
cargo test -p m20-websockets-tests --features mine ex3_
cargo run  -p m20-websockets-solution -- serve
cargo test -p m20-websockets-tests                     # always green
```

## Talk to the Server by Hand

Install [websocat](https://github.com/vi/websocat) (`brew install websocat` or
`cargo install websocat`), start `-- serve`, then in two terminals:

```bash
websocat 'ws://127.0.0.1:3000/ws/chat?name=alice'
{"type":"join","room":"rust"}
{"type":"say","text":"hello from alice"}
```

```bash
websocat 'ws://127.0.0.1:3000/ws/chat?name=bob'
{"type":"join","room":"rust"}
```

Notifications:

```bash
websocat 'ws://127.0.0.1:3000/ws/notifications?user=bob'      # terminal 1
curl -X POST localhost:3000/notify/bob -H 'content-type: application/json' -d '{"title":"hi","body":"from curl"}'
```

Dashboard: `websocat 'ws://127.0.0.1:3000/ws/dashboard?every_ms=500'`.

In a browser console:

```js
const ws = new WebSocket("ws://127.0.0.1:3000/ws/chat?name=web");
ws.onmessage = (e) => console.log(JSON.parse(e.data));
ws.onopen = () => ws.send(JSON.stringify({ type: "join", room: "rust" }));
```

## If You Get Stuck

1. **The handshake fails with 404** — the route must be a `get(...)`: the upgrade is an HTTP GET.
2. **A test hangs** — a `select!` loop isn't exiting when `socket.recv()` returns `None` or `Close`; check every branch can `break`.
3. **Messages arrive out of order between clients** — messages on *different* connections are handled concurrently; tests and clients must wait for confirmations (see the demo).
4. **`Lagged` errors** — a receiver fell behind its channel's capacity; decide whether to skip (chat) or resend state (document).
5. **Heartbeat closes healthy connections** — update `last_seen` on *every* incoming frame, including Pong.
6. Compare against `solution/src/` — same file and function names.
