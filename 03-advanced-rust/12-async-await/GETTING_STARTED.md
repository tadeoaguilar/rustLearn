# Getting Started with 12 · Async/Await

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m12-async-await-solution -- 2
```

No internet connection is needed: the HTTP exercises talk to a mock API that
the program starts on a random local port.

## What Is Already Here

```
12-async-await/
├── README.md               # The concepts
├── exercises.md            # The work: 8 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m12-async-await)
│   ├── Cargo.toml          #   tokio, tokio-stream, futures, reqwest, axum, serde ready
│   └── src/
│       ├── main.rs         #   #[tokio::main] runner: `-- 1` … `-- 8`, `-- bonus`, `-- serve`
│       ├── mock_api.rs     #   the local test API (complete -- not an exercise)
│       └── ex01_basics.rs … bonus_task_queue.rs   (bodies are todo!())
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m12-async-await-solution)
│
└── tests/                  # ← 20 tests (m12-async-await-tests), paused-clock timers
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m12-async-await -- 1

# Check your work
cargo test -p m12-async-await-tests --features mine

# One exercise's tests (names start ex1_ … ex8_, bonus_)
cargo test -p m12-async-await-tests --features mine ex5_

# The reference solution
cargo run -p m12-async-await-solution -- 1       # async fns, laziness
cargo run -p m12-async-await-solution -- 2       # join!, select!, timeout (takes ~7s: real timers)
cargo run -p m12-async-await-solution -- 3       # tokio::fs
cargo run -p m12-async-await-solution -- 4       # reqwest against the mock API
cargo run -p m12-async-await-solution -- 5       # channels
cargo run -p m12-async-await-solution -- 6       # spawn, abort, JoinSet, Semaphore
cargo run -p m12-async-await-solution -- 7       # axum: requests against itself
cargo run -p m12-async-await-solution -- 8       # streams
cargo run -p m12-async-await-solution -- bonus   # task queue
cargo run -p m12-async-await-solution -- all     # everything

cargo test -p m12-async-await-tests              # always green, under a second
```

## Exercise 7: Talk to the Server Yourself

```bash
cargo run -p m12-async-await-solution -- serve
```

In another terminal:

```bash
curl localhost:3000/
curl -X POST localhost:3000/todos -H 'content-type: application/json' -d '{"title":"learn async"}'
curl -i localhost:3000/todos/1          # note the x-request-count header
curl -i -X PATCH localhost:3000/todos/1
curl -i localhost:3000/todos/99         # 404 with a JSON error body
```

Ctrl-C stops it gracefully (`with_graceful_shutdown`).

## Writing Async Tests

```rust
#[tokio::test(start_paused = true)]   // timers only: instant, exact
async fn my_timer_test() { ... }

#[tokio::test]                        // real I/O (files, sockets)
async fn my_network_test() { ... }
```

The tests crate enables Tokio's `test-util` feature for `start_paused`.

## If You Get Stuck

1. **`future cannot be sent between threads safely`** — something non-`Send` (an `Rc`, a `std::sync::MutexGuard`) is alive across an `.await` inside a spawned task. Drop it before the `.await`, or use `Arc` / `tokio::sync::Mutex`.
2. **`borrowed value does not live long enough` with `tokio::spawn`** — spawned futures must be `'static`; move owned data (or `Arc` clones) in with `async move`.
3. **The program hangs at `rx.recv()`** — a `Sender` is still alive somewhere (often the original you cloned from). Drop it.
4. **`unused implementer of Future that must be used`** — you forgot `.await`.
5. **`` `IntervalStream` is not an iterator ``** — you need `use tokio_stream::StreamExt;` and `.await` on `collect()`/`next()`.
6. Compare against `solution/src/` — same file and function names.
