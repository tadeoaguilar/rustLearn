# 12 · Async/Await

## Overview

Async Rust lets one thread juggle thousands of tasks that spend most of their
time **waiting** — on sockets, timers, files. An `async fn` returns a *future*:
a description of work that does nothing until it is awaited. `.await` yields
the thread to the runtime (Tokio, here) instead of blocking it, and the
runtime resumes the task when its I/O is ready.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Async functions | Futures are lazy; `?` works the same in async |
| 2 | Concurrency | `join!`, `join_all`, `select!`, `timeout` — total time = longest, not sum |
| 3 | File I/O | `tokio::fs`, line-by-line reading, concurrent reads |
| 4 | HTTP client | `reqwest`, reusable clients, retry with backoff — against a local mock |
| 5 | Channels | `mpsc`, `broadcast`, `oneshot` request/response |
| 6 | Tasks | `spawn`, abort vs cooperative cancellation, `JoinSet`, `Semaphore`, `spawn_blocking` |
| 7 | Web server | axum routes, shared state, `IntoResponse` errors, middleware |
| 8 | Streams | Interval streams, adaptors, `merge`, `unfold` |
| Bonus | Task queue | Worker pool with priority, retries, results and rate limiting |

## Key Concepts

### Concurrency without threads

```rust
let r1 = task_one().await;          // 1s
let r2 = task_two().await;          // then 2s  -> 3s total
let (r1, r2) = tokio::join!(task_one(), task_two());   // 2s total
```

`join!` doesn't spawn anything: it polls both futures on the same task, and
while one waits on its timer the other runs.

### `spawn` needs `Send + 'static`

`tokio::spawn` hands the future to the runtime to run independently, maybe on
another thread and maybe after your function returns. So it can't borrow
locals (`'static`) and must be safe to move between threads (`Send`). Clone
`Arc`s in, move owned data in, and get results out through the `JoinHandle`
or a channel.

### Never block the runtime

| Don't (blocks the worker thread) | Do |
|---|---|
| `std::thread::sleep` | `tokio::time::sleep(..).await` |
| `std::fs::read_to_string` in a hot path | `tokio::fs::read_to_string(..).await` |
| a long CPU loop | `tokio::task::spawn_blocking(..)` |
| `std::sync::Mutex` held across `.await` | `tokio::sync::Mutex`, or don't hold it across `.await` |

### Cancellation = dropping a future

`select!` cancels the losing branches by dropping them; `JoinHandle::abort`
cancels a spawned task at its next `.await`. Code after an `.await` may never
run — so put cleanup in `Drop` or use cooperative shutdown signals (Exercise 6).

### Testing time without waiting

```rust
#[tokio::test(start_paused = true)]
async fn concurrent_takes_two_seconds() { ... }
```

With a paused clock, Tokio jumps straight to the next timer whenever all tasks
are idle. This module's timer tests take microseconds and measure *exact*
virtual durations (use `tokio::time::Instant`, not `std::time::Instant`).

## Common Pitfalls

1. **Forgetting `.await`** — the future is created and dropped; nothing runs (there's a warning: `unused implementor of Future`)
2. **Forgetting to drop the last `Sender`** — `recv()` waits forever
3. **Blocking calls inside async code** — every task on that thread stalls
4. **A new `reqwest::Client` per request** — loses connection pooling
5. **Not checking `error_for_status()`** — a 404 page reaches `.json()` and fails confusingly
6. **Retrying everything** — retry timeouts and 5xx, never 4xx

## Running This Module

```bash
cargo run  -p m12-async-await -- 2                       # your code
cargo test -p m12-async-await-tests --features mine      # test your code
cargo run  -p m12-async-await-solution -- all            # the reference solution (~13s of real timers)
cargo run  -p m12-async-await-solution -- serve          # Exercise 7's server on 127.0.0.1:3000
cargo test -p m12-async-await-tests                      # 20 tests against the solution (<1s)
```

## Notes on `exercises.md`

- **Bonus**: `rx.clone()` doesn't compile — `tokio::sync::mpsc::Receiver` is
  single-consumer and not `Clone` (`error[E0599]: no method named 'clone'`).
  The minimal fix shares one receiver behind `Arc<tokio::sync::Mutex<_>>`;
  it's in `bonus_task_queue::shared_receiver_pool`. The full queue (priority,
  retry, results, rate limit) uses a shared `BinaryHeap` instead.
- **Exercise 4** calls `jsonplaceholder.typicode.com`. The solution and tests
  call `mock_api`, an axum server started in-process on a random port, which
  serves the same `/posts/{id}` shape plus `/flaky` (fails twice) and `/slow`
  (2 s) for practising retries and timeouts. Every function takes a `base` URL,
  so pointing it at the real service works too. The struct field `userId`
  triggers the `non_snake_case` lint; `#[serde(rename_all = "camelCase")]`
  maps it to `user_id`.
- **Exercise 4/7 versions**: the exercise lists `reqwest 0.11` and `axum 0.7`.
  The workspace uses reqwest 0.12 and axum 0.8; the only visible change is
  axum's path syntax, `/todos/{id}` instead of `/todos/:id`.
- **Exercise 3** writes `test.txt` into the current directory; the solution
  uses a temporary directory.
- **Exercise 6** calls a function that sleeps "CPU-intensive". Real CPU work
  belongs in `spawn_blocking`; the solution adds an example.
- **Exercise 2, Task 1** ("10 tasks with `join!`"): `join!` takes a fixed list
  of futures; for ten, `futures::future::join_all` is the practical tool.

## Next

[13 · Macros](../13-macros/)
