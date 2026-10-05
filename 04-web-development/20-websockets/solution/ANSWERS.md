# Answers · 20 WebSockets

## Exercise 1

**What does the HTTP exchange look like before the first message?**

```text
GET /ws/echo HTTP/1.1
Host: 127.0.0.1:3000
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13

HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

`Sec-WebSocket-Accept` is base64(SHA-1(key + a fixed GUID)) — proof that the
server really speaks WebSocket. After the 101, the TCP connection carries
WebSocket frames instead of HTTP. Because it starts as HTTP, normal middleware
(auth, logging) and rejections (our 400 for a missing `?name=`) work before
the upgrade.

**Who answers pings?** The library. A Ping frame must be answered with a Pong
carrying the same payload; axum and tungstenite do it automatically — but only
while you're *reading* the socket. That's why a client that stops reading
looks dead to the server's heartbeat (Exercise 5's test relies on it).

## Exercise 6: How do real editors merge concurrent edits?

- **Operational Transformation (OT)**: edits are operations ("insert 'x' at
  5"); when two arrive concurrently, the server transforms one against the
  other so both apply in either order. Google Docs uses OT.
- **CRDTs** (Conflict-free Replicated Data Types): each character gets a
  unique, ordered identity, so concurrent inserts and deletes commute by
  construction, even peer-to-peer and offline. Figma, Zed and many local-first
  apps use CRDTs; in Rust see `yrs` (Yjs) and `automerge`.

Both send small operations instead of whole documents, which also fixes the
bandwidth problem of our version.

## Bonus: Why add random jitter to the backoff?

When a server restarts, every client disconnects at the same moment. With pure
exponential backoff they all wait exactly 1 s, 2 s, 4 s … and reconnect in
synchronised waves — a "thundering herd" that can knock the server over again.
Randomising each delay (e.g. a random value between 0 and the computed delay,
"full jitter") spreads reconnections out evenly.
