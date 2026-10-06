# 32 · Network Programming

## Overview

Under every framework are sockets: a listener that accepts connections,
streams you read and write, datagrams you send and receive. This module
works at that level -- an echo server two ways, an HTTP/1.1 server with no
HTTP library, a proxy and load balancer, a chat server with its own binary
protocol, UDP ping and a DNS message codec -- so that what axum, hyper and
tonic do on top stops being magic.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Echo | Accept/read/write; a thread per client vs a task per client |
| 2 | HTTP/1.1 | Parsing with limits; keep-alive; routing; path traversal |
| 3 | Proxy, load balancer | Bidirectional copy; health checks; failover |
| 4 | Chat | Framing; binary encoding; broadcast with `select!` |
| 5 | UDP ping | Datagrams; network byte order; loss and RTT |
| Bonus | DNS | A real binary protocol with compression pointers |

## Key Concepts

### TCP is a stream

```
client writes:  "HELLO"  "WORLD"
server reads:   "HEL"  "LOWOR"  "LD"        <- any split is legal
```

There are no messages in TCP, only bytes. Protocols mark boundaries
themselves: a delimiter (lines in Exercise 1, the blank line ending HTTP
headers), a length (`Content-Length`, the 4-byte frame header of Exercise 4).

### UDP is datagrams

Each `send_to` is one datagram, delivered whole or not at all, maybe out of
order. Reliability -- sequence numbers, timeouts, retries -- is up to you
(or QUIC, which builds it on UDP).

### Byte order

`u16::to_be_bytes` / `from_be_bytes`: network protocols are big-endian;
x86 and ARM are little-endian. Never send `transmute`d structs.

### Async I/O

```rust
loop {
    let (stream, _) = listener.accept().await?;
    tokio::spawn(handle(stream));           // a task per connection
}
```

`into_split()` gives independent read and write halves; `select!` waits on
several things at once (a frame from this client, a broadcast from others).

### Never trust the peer

Every length is a limit to check (`MAX_HEAD`, `MAX_BODY`, `MAX_FRAME`), every
path a traversal attempt, every pointer a potential loop.

## Common Pitfalls

1. **Assuming one `read` = one message** -- frame your messages
2. **Unbounded reads** -- `read_line` on an attacker's 10 GB line; use `take`
3. **Trusting `Content-Length`** -- allocate only after checking the limit
4. **Path checks before decoding** -- `%2e%2e` is `..`
5. **Blocking calls in async code** -- `std::fs`/`std::net` stall the runtime thread
6. **Forgetting `flush`/`shutdown`** -- the peer waits for bytes still in a buffer
7. **Port collisions in tests** -- bind to port 0

## Running This Module

```bash
cargo run  -p m32-network-programming -- 1                     # your code (1-5, bonus, all)
cargo test -p m32-network-programming-tests --features mine    # test your code
cargo run  -p m32-network-programming-solution -- all          # every server and client, on local ports
cargo run  -p m32-network-programming-solution -- http 8080 .  # a real server: curl localhost:8080/echo?msg=hi
cargo test -p m32-network-programming-tests                    # 25 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a TCP echo server, a simple HTTP server, a
proxy server, a custom protocol).

- **TLS** isn't repeated here: module 25 terminates TLS with rustls (and
  mutual TLS); wrapping Exercise 1's server in `tokio_rustls` is the same code.
- **HTTP/2, QUIC and connection pooling** from the topic list are left to
  hyper and quinn; Exercise 2 covers HTTP/1.1 without chunked transfer encoding.
- **No test uses the internet**: the DNS bonus parses hand-built packets.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[33 · Embedded Rust](../33-embedded-rust/)
