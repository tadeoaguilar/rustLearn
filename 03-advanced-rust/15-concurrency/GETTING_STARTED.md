# Getting Started with 15 · Concurrency

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m15-concurrency-solution --release -- all
```

`--release` matters here: debug builds make the timing comparisons meaningless.

## What Is Already Here

```
15-concurrency/
├── README.md               # The concepts
├── exercises.md            # The work: 8 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m15-concurrency)
│   ├── Cargo.toml          #   rayon and crossbeam already added
│   └── src/ex01_threads.rs … bonus_philosophers.rs   (bodies are todo!())
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m15-concurrency-solution)
│   ├── ANSWERS.md
│   └── src/
│
└── tests/                  # ← 25 tests (m15-concurrency-tests), each deadlock-protected
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m15-concurrency -- 1

# Check your work
cargo test -p m15-concurrency-tests --features mine

# One exercise's tests (names start ex1_ … ex8_, bonus_)
cargo test -p m15-concurrency-tests --features mine ex3_

# The reference solution
cargo run -p m15-concurrency-solution -- 1                 # threads, scope
cargo run -p m15-concurrency-solution -- 2                 # channels
cargo run -p m15-concurrency-solution -- 3                 # bank, BlockingQueue
cargo run -p m15-concurrency-solution --release -- 4       # atomics (timings!)
cargo run -p m15-concurrency-solution -- 5                 # thread pool
cargo run -p m15-concurrency-solution --release -- 6       # rayon (timings!)
cargo run -p m15-concurrency-solution -- 7                 # crawler: 1 vs 4 vs 8 workers
cargo run -p m15-concurrency-solution -- 8                 # lock-free stack
cargo run -p m15-concurrency-solution -- bonus             # dining philosophers

cargo test -p m15-concurrency-tests                        # always green
```

## When a Test Hangs (or Fails With "deadlock?")

Tests that could deadlock run under a watchdog:

```text
did not finish within 5s -- deadlock or lost wake-up?
```

Common causes:
1. A lock taken in different orders in different places (Exercise 3, Bonus).
2. A `Condvar` waited on with `if` instead of `while`, or a missing `notify_*`.
3. A `SpinGuard` whose `Drop` doesn't release the lock.
4. A `Sender` still alive, so a receiving loop never ends.
5. The crawler stopping only when the queue is empty, not when *in-flight work* is zero (or never stopping).

To investigate, run one test alone and print from inside the threads:

```bash
cargo test -p m15-concurrency-tests --features mine ex3_push_blocks -- --nocapture --test-threads=1
```

## Finding Data Races in `unsafe` Code

The type system rules out data races in safe code. In `unsafe` code (the spin
lock, the lock-free stack) use the thread sanitizer or Miri:

```bash
rustup +nightly component add miri
cargo +nightly miri test -p m15-concurrency-tests -- ex4_spin ex8_stack_is_lifo
```

## If You Get Stuck

1. **`` `Rc<...>` cannot be sent between threads safely ``** — use `Arc`.
2. **`borrowed value does not live long enough` with `thread::spawn`** — move owned data in, or use `thread::scope`.
3. **`` `Receiver<...>` cannot be shared between threads safely ``** — wrap it: `Arc<Mutex<Receiver<_>>>`, or use `crossbeam::channel` (its receivers clone).
4. **`` `UnsafeCell<...>` cannot be shared between threads safely ``** — you need `unsafe impl<T: Send> Sync for SpinLock<T> {}`, with a `SAFETY` argument.
5. Compare against `solution/src/` — same file and function names.
