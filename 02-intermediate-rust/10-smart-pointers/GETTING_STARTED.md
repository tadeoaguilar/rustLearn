# Getting Started with 10 · Smart Pointers

## Quick Start

All commands run from the **repository root**.

```bash
cargo run -p m10-smart-pointers-solution -- all
```

## What Is Already Here

```
10-smart-pointers/
├── README.md               # The concepts
├── exercises.md            # The work: 7 exercises plus a bonus
├── GETTING_STARTED.md      # This file
│
├── exercise/               # ← YOUR WORKSPACE (package m10-smart-pointers)
│   └── src/ex01_box.rs … bonus_cow.rs
│                           #   types declared; bodies are todo!() -- except
│                           #   Drop::drop, which is empty (see README)
│
├── solution/               # ← REFERENCE IMPLEMENTATION (m10-smart-pointers-solution)
│   ├── ANSWERS.md
│   └── src/
│
└── tests/                  # ← 21 tests (m10-smart-pointers-tests)
```

## The Commands You Need

```bash
# Run your own work
cargo run -p m10-smart-pointers -- 1

# Check your work
cargo test -p m10-smart-pointers-tests --features mine

# One exercise's tests (names start ex1_ … ex7_, bonus_)
cargo test -p m10-smart-pointers-tests --features mine ex4_

# The reference solution
cargo run -p m10-smart-pointers-solution -- 1       # Box, recursive types
cargo run -p m10-smart-pointers-solution -- 2       # Linked list
cargo run -p m10-smart-pointers-solution -- 3       # Deref, Drop, drop order
cargo run -p m10-smart-pointers-solution -- 4       # Rc/Weak tree
cargo run -p m10-smart-pointers-solution -- 5       # RefCell, Cell
cargo run -p m10-smart-pointers-solution -- 6       # Graph and a leak
cargo run -p m10-smart-pointers-solution -- 7       # Arc<Mutex> across threads
cargo run -p m10-smart-pointers-solution -- bonus   # Cow
cargo run -p m10-smart-pointers-solution -- all     # Everything

cargo test -p m10-smart-pointers-tests              # always green
```

## See the Stack Overflow for Yourself

Implement `push` in Exercise 2 but leave `Drop` empty, then:

```bash
cargo test -p m10-smart-pointers-tests --features mine ex2_dropping
```

```text
thread '...' has overflowed its stack
fatal runtime error: stack overflow
```

The whole test binary dies — no other tests report. That's why the test exists.

## If You Get Stuck

1. **`cannot move out of ... which is behind a mutable reference`** — use `self.head.take()` (or `std::mem::replace`).
2. **`already borrowed: BorrowMutError`** — two `RefCell` borrows overlap; end the first (`drop(guard)` or a smaller scope) before the second.
3. **`Rc<...> cannot be sent between threads safely`** — use `Arc`, and `Mutex` instead of `RefCell`.
4. **A test hangs in Exercise 7** — a deadlock: you're locking twice in the same thread (e.g. calling `self.get()` while holding the lock).
5. **Weak `upgrade()` is always `None`** — nothing holds a strong `Rc` to the target any more.
6. Compare against `solution/src/` — same file and function names.
