# Getting Started with 32 · Network Programming

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m32-network-programming-solution -- all
```

starts each server on a free local port and talks to it: echo with 100
concurrent clients, a series of raw HTTP requests (including a path
traversal attempt and garbage), a proxy counting bytes and a balancer
skipping a dead upstream, two chat clients, UDP pings over a lossy server,
and a DNS response parsed from bytes.

## What Is Already Here

```
32-network-programming/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m32-network-programming)
│   ├── ex01_echo.rs           #   Ex 1  (echo_client provided)
│   ├── ex02_http.rs           #   Ex 2  (Response builders, reason() provided)
│   ├── ex03_proxy.rs          #   Ex 3
│   ├── ex04_chat.rs           #   Ex 4  (Room, ChatClient, put_str, Cursor provided)
│   ├── ex05_udp.rs            #   Ex 5
│   └── bonus_dns.rs           #   bonus
├── solution/                  # ← REFERENCE (m32-network-programming-solution) + ANSWERS.md
└── tests/                     # ← 25 tests (m32-network-programming-tests)
```

## The Commands You Need

```bash
cargo test -p m32-network-programming-tests --features mine ex2_
cargo test -p m32-network-programming-tests --features mine
cargo test -p m32-network-programming-tests                    # the solution: always green
```

## Try It With Real Tools (Optional)

```bash
cargo run -p m32-network-programming -- http 8080 .             # your server, this directory as /static
curl -v 'localhost:8080/echo?msg=hello+world'
curl -d 'shout' localhost:8080/upper
curl --path-as-is localhost:8080/static/../Cargo.toml           # 403
printf 'GET / HTTP/1.1\r\n\r\nGET / HTTP/1.1\r\nConnection: close\r\n\r\n' | nc localhost 8080
```

(`nc` and `curl` ship with macOS and most Linux distributions; they aren't
needed for the tests.)

## If You Get Stuck

1. **A test "timed out"** -- usually a missing `flush`/`shutdown`, or a server waiting for a line the client never ends with `\n`.
2. **The HTTP client hangs after a response** -- `Content-Length` must match the body exactly, and `Connection: close` must really close.
3. **reqwest reports "connection closed before message completed"** -- the server closed a keep-alive connection; only close when asked (or on errors).
4. **Chat messages arrive out of order or missing** -- subscribe to the broadcast before sending `Welcome`; handle `RecvError::Lagged`.
5. **`ping` counts the wrong pongs** -- compare the sequence number; a late pong for the previous ping isn't this one's.
6. **UDP `recv` returns `ConnectionRefused`** -- on a connected socket, an ICMP "port unreachable" from an earlier send surfaces on the next call; treat it as a lost packet.
7. Compare with `solution/src/` -- same file and function names.
