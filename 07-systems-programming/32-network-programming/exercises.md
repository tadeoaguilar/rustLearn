# Exercises: Network Programming

Every networked program sits on two primitives: **TCP**, a reliable,
ordered byte stream between two endpoints, and **UDP**, independent
datagrams that may be lost, duplicated or reordered. Protocols are the
conventions on top: where a message ends, what its bytes mean, who speaks
when. This module writes servers and clients for five of them with
`std::net` and `tokio::net`.

**Setup**: tokio only (reqwest in the tests, to check the HTTP server
against a real client). Everything binds to `127.0.0.1:0` -- the OS picks a
free port -- so tests run in parallel without a network. Each async test has
a 10-second watchdog.

---

## Exercise 1: Echo Servers

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Accept connections, read lines, write replies
- Compare a thread per client with a task per client

Protocol: every line comes back as `echo: <line>`; the line `bye` closes the
connection.

1. `reply(line)` (`None` for `bye`; strip `\r\n`).
2. `handle_blocking` / `serve_blocking`: `std::net`, a thread per client;
   increment `served` when a client finishes.
3. `handle_async` / `serve_async`: `tokio::net`, a task per client.
   (`echo_client` is provided.)

**Question**: the tests open 200 connections at once against the async
server. What would 10,000 cost with threads?

---

## Exercise 2: HTTP/1.1 from Scratch

**Difficulty**: Hard
**Time**: 3 hours

**Learning Objectives**:
- Parse a text protocol with limits
- Implement keep-alive and `Content-Length` bodies
- Serve files without letting clients escape the directory

1. `percent_decode(s, plus_is_space)`, `split_target` (path and query pairs).
2. `parse_head`: request line (`METHOD /target HTTP/1.x`, uppercase
   method), headers (`Name: value`, trimmed; no spaces in names) -- else
   `Malformed`. `Request::header` (case-insensitive), `query_param`,
   `keep_alive` (HTTP/1.1: unless `Connection: close`; HTTP/1.0: only with
   `Connection: keep-alive`).
3. `read_request`: lines until the blank line (bare `\n` tolerated), at most
   `MAX_HEAD` bytes (`HeadTooLarge`), then `Content-Length` bytes (at most
   `MAX_BODY`: `BodyTooLarge`). `Ok(None)` on a clean close before a request.
4. `Response::to_bytes(keep_alive)`: status line, headers, `Content-Length`,
   `Connection`, blank line, body. `HttpError::response`: 400 / 431 / 413.
5. `route`: `GET /`, `GET /echo?msg=`, `POST /upper`, `GET /static/<file>`
   (403 for `..` or absolute paths -- *after* decoding; `content_type` by
   extension; 404 if missing); 405 with `Allow` for a wrong method; 404.
6. `handle_connection` (requests until close, keep-alive or an error),
   `serve`.

**Question**: why must the path be checked *after* percent-decoding?

---

## Exercise 3: A Proxy and a Load Balancer

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Relay bytes in both directions
- Spread connections over healthy upstreams

1. `pipe` (hint: `tokio::io::copy_bidirectional`), `serve_proxy`: count
   connections and bytes in `Stats`.
2. `Balancer::new` (all healthy), `set_healthy`, `is_healthy`, `pick`:
   round-robin over healthy upstreams, `None` if none.
3. `check_all`: healthy iff a TCP connection opens within the timeout;
   `health_loop`.
4. `serve_balanced`: pick per connection; if the upstream refuses, mark it
   unhealthy and try the next.

---

## Exercise 4: A Framed Protocol -- Chat

**Difficulty**: Hard
**Time**: 2.5 hours

**Learning Objectives**:
- Frame messages on a byte stream
- Encode messages as compact binary
- Broadcast to many clients with `tokio::sync::broadcast` and `select!`

Frames: `[u32 BE length][payload]`, payloads at most `MAX_FRAME`. Payload:
a tag byte, then fields; strings are `[u16 BE length][UTF-8]`
(`put_str` and `Cursor` are provided).

| Tag | Message |
|---|---|
| 1 / 2 / 3 | `Join{name}` / `Say{text}` / `Leave` |
| 10 | `Welcome{users}` (`[u16 count]` then strings) |
| 11 / 12 | `Joined{name}` / `Left{name}` |
| 13 | `Message{from, text}` |
| 14 | `Error{reason}` |

1. `encode`/`decode` for both enums (`UnknownTag`, `Truncated`, `BadUtf8`).
2. `write_frame`, `read_frame` (`None` on a clean close, `TooLarge`, `Truncated`).
3. `valid_name`; `handle_client`: the first message must be `Join` (else
   `Error{"join first"}`), with a valid (`"invalid name"`) and unused
   (`"name taken"`) name. Send `Welcome` (sorted users, including the new
   one), broadcast `Joined` to the others, then relay `Say` as `Message` to
   everyone (sender included) until `Leave` or disconnect; then `Left`.

**Question**: why subscribe to the broadcast channel *before* sending
`Welcome` and announcing `Joined`?

---

## Exercise 5: UDP Ping

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Encode fields in network byte order
- Detect loss with sequence numbers and timeouts

Packets: `[seq: u16 BE][sent: u64 BE]`, exactly 10 bytes.

1. `Ping::encode` / `decode`.
2. `serve_pong`: echo valid packets back; with `drop_every: Some(k)`, drop
   every k-th.
3. `PingStats`: `loss_percent`, `min`, `avg`, `max`.
4. `ping(server, count, interval, timeout)`: wait for *this* sequence
   number's pong (ignore stale ones) up to `timeout`.

---

## Bonus: DNS Messages

**Difficulty**: Medium
**Time**: 1 hour

In `bonus_dns.rs`: `encode_name`, `build_query` (an A query with recursion
desired), `decode_name` (with compression pointers -- which must point
*backwards*, ruling out loops), `parse_response`, `a_records`. Tested
against hand-built packets; no network.
