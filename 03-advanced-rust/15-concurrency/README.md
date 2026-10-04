# 15 · Concurrency

## Overview

Rust's "fearless concurrency" is a precise claim: **data races are compile
errors**. Two marker traits, checked by the compiler, decide what may cross
thread boundaries. What the type system can't catch — deadlocks, lost
wake-ups, logic races, contention — is still your job, and that's what this
module practises: threads, channels, locks, atomics, a thread pool, rayon, a
crawler that has to know when it's finished, and a lock-free stack.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Threads | `spawn` + `join`, named threads, `thread::scope` to borrow |
| 2 | Channels | Fan-out/fan-in pipelines; when a channel ends; backpressure |
| 3 | Shared state | Per-account locks, deadlock-free lock ordering, `Condvar` |
| 4 | Atomics | `Relaxed` vs `Acquire`/`Release`; CAS loops; a spin lock |
| 5 | Thread pool | Graceful shutdown in `Drop`; surviving panicking jobs |
| 6 | Rayon | `par_iter`; a parallel file processor |
| 7 | Crawler | Dynamic work, a shared visited set, termination detection |
| 8 | Lock-free | A Treiber stack with crossbeam-epoch; `ArrayQueue` |
| Bonus | Philosophers | Deadlock and resource ordering |

## Key Concepts

### `Send` and `Sync`

| Type | `Send` | `Sync` | Why |
|---|---|---|---|
| `i32`, `String`, `Vec<T: Send>` | ✓ | ✓ | owned, no shared mutation |
| `Rc<T>` | ✗ | ✗ | non-atomic reference count |
| `Arc<T>` | if `T: Send + Sync` | same | atomic count |
| `RefCell<T>` | ✓ | ✗ | non-atomic borrow flag |
| `Mutex<T>` | if `T: Send` | if `T: Send` | the lock makes sharing safe |
| `*mut T` | ✗ | ✗ | the compiler can't know; you decide with `unsafe impl` |

### Choosing a tool

| Need | Use |
|---|---|
| Independent tasks, results at the end | `thread::spawn` / `thread::scope` + `join` |
| Hand work along a pipeline | channels (`mpsc`, `crossbeam::channel` for many consumers) |
| A shared counter or flag | atomics |
| A shared structure, mostly writes | `Mutex` |
| A shared structure, mostly reads | `RwLock` |
| Wait until a condition holds | `Condvar` (in a loop), or a channel |
| CPU-bound work over a collection | rayon `par_iter` |

### Deadlocks need a cycle — so forbid cycles

Thread A holds lock 1 and wants 2; thread B holds 2 and wants 1. Neither moves.
If every thread acquires locks in **one global order** (lower index first),
no cycle can form. The bank (Exercise 3) and the philosophers (Bonus) use the
same fix.

### Knowing when parallel work is done

A crawler's queue can be empty while a worker is still fetching a page that
will add ten more links. Exercise 7 counts work that is *queued or in
progress* in one atomic: new links are counted **before** the finished page is
uncounted, so the count reaches zero exactly once — when everything is done.

### Memory ordering, briefly

`Relaxed` makes one operation atomic and nothing more — fine for counters read
after `join`. A lock needs `Acquire` when taking it and `Release` when giving
it back, so the data written inside the critical section is visible to the
next owner. `SeqCst` adds a single global order; it's the safe default when
unsure. Read *Rust Atomics and Locks* before going further.

### Lock-free ≠ easy

The CAS loop is ten lines. Freeing popped nodes safely — while another thread
may still be reading them — is the hard part; epoch-based reclamation
(`crossbeam::epoch`) or hazard pointers solve it. In production, use
`crossbeam::queue`, `dashmap`, or a `Mutex` — they're very good.

## Common Pitfalls

1. **Holding a `MutexGuard` across a long operation** — everyone else waits; keep critical sections tiny
2. **`if` instead of `while` around `Condvar::wait`** — spurious wake-ups and stolen items
3. **Locking in different orders in different places** — deadlock under load, never in your tests
4. **Forgetting to drop the last `Sender`** — receivers wait forever
5. **`Relaxed` on a flag that publishes data** — the reader sees the flag but stale data
6. **Benchmarking in debug mode** — use `--release`
7. **Assuming parallel = faster** — tiny workloads lose to the overhead

## Running This Module

```bash
cargo run  -p m15-concurrency -- 3                               # your code
cargo test -p m15-concurrency-tests --features mine              # test your code (watchdog-protected)
cargo run  -p m15-concurrency-solution --release -- all          # the reference solution, real timings
cargo test -p m15-concurrency-tests                              # 25 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of the others and the exercise list in
[the phase README](../README.md) (a parallel file processor, a concurrent web
crawler, a thread pool, a lock-free queue).

- **No network**: the crawler crawls `FakeWeb`, an in-memory graph with a
  configurable latency, so results and timings are deterministic.
- **Watchdog tests**: every test that could deadlock runs under a `deadline`
  helper that fails after a few seconds instead of hanging the test runner.
- **Lock-free stack**: written with `crossbeam::epoch` rather than raw
  `AtomicPtr`, because a raw Treiber stack that frees nodes immediately is a
  use-after-free waiting to happen.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

Phase 3 is complete — and with it, everything in this repository that has
code. Continue with one of the specialisations in
[Phase 4 · Web Development](../../04-web-development/) and beyond, or build the
[CRUD API project](../../10-crud-api-project/).
