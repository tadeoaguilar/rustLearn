# 10 · Smart Pointers

## Overview

References (`&T`, `&mut T`) borrow; they never own. Sometimes you need more:
a value on the heap (`Box`), several owners (`Rc`, `Arc`), mutation through a
shared reference (`Cell`, `RefCell`, `Mutex`), or a pointer that doesn't keep
its target alive (`Weak`). Those are smart pointers — structs that implement
`Deref` (so they act like references) and usually `Drop` (so they clean up).

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | `Box` | Recursive types need indirection; an expression tree |
| 2 | Linked list | `Option<Box<Node>>`, `take`, `as_deref`, a non-recursive `Drop` |
| 3 | `Deref` / `Drop` | Deref coercion; the exact order things are dropped in |
| 4 | `Rc` + `Weak` | A tree whose children know their parent without owning it |
| 5 | `RefCell` / `Cell` | Interior mutability; runtime borrow errors |
| 6 | `Rc<RefCell<T>>` | A mutable graph; watching a reference cycle leak |
| 7 | `Arc<Mutex<T>>` | A cache shared across threads; compute-once; poisoning |
| Bonus | `Cow` | Borrow when you can, allocate when you must |

## Key Concepts

### Which pointer when?

| You need | Single thread | Multiple threads |
|---|---|---|
| Heap allocation, one owner | `Box<T>` | `Box<T>` |
| Several owners | `Rc<T>` | `Arc<T>` |
| Mutate through `&self` | `Cell<T>` (Copy) / `RefCell<T>` | `Mutex<T>` / `RwLock<T>` / atomics |
| Several owners *and* mutation | `Rc<RefCell<T>>` | `Arc<Mutex<T>>` |
| Point without owning | `rc::Weak<T>` | `sync::Weak<T>` |

The compiler enforces the right column: `Rc` and `RefCell` aren't `Send`/`Sync`,
so `thread::spawn` rejects them.

### Recursive drops can overflow the stack

```rust
struct Node<T> { value: T, next: Option<Box<Node<T>>> }
```

Dropping the head drops `next`, which drops *its* `next`… one stack frame per
node. A million nodes is a stack overflow. Exercise 2's `Drop` unlinks nodes in
a loop instead, and a test with 1,000,000 nodes proves it.

### `Rc` cycles leak

Two `Rc`s pointing at each other each keep the other's strong count at 1 —
forever. Rust's memory safety guarantees don't include "no leaks". The fix is
structural: decide which direction *owns* (parent → child, network → person)
and make the other direction `Weak`.

### `RefCell` moves the borrow check to runtime

```rust
let a = cell.borrow_mut();
let b = cell.borrow_mut();   // compiles, then panics: "already borrowed"
```

Same rules as references, checked later. Keep `RefCell` borrows short (don't
hold a `borrow()` across a call that might `borrow_mut()`), or use
`try_borrow_mut()` where a conflict is possible.

### Drop order

- locals: **reverse** declaration order
- struct fields and tuple elements: **declaration** order
- `let _ = expr;` drops immediately; `let _x = expr;` lives to the end of scope

## Common Pitfalls

1. **`Rc<RefCell<T>>` everywhere** — often a sign the ownership design isn't settled yet
2. **Holding a `RefCell`/`Mutex` borrow too long** — runtime panic or deadlock
3. **Strong references in both directions** — a leak; one side must be `Weak`
4. **`#[derive(Clone)]` on a handle type** — it adds `T: Clone` bounds that cloning an `Arc` doesn't need
5. **`.lock().unwrap()` everywhere** — decide what a poisoned lock means for your data

## Running This Module

```bash
cargo run  -p m10-smart-pointers -- 4                      # your code, exercise 4
cargo test -p m10-smart-pointers-tests --features mine     # test your code
cargo run  -p m10-smart-pointers-solution -- all           # the reference solution
cargo test -p m10-smart-pointers-tests                     # 21 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of the others and the topics and exercise list in
[the phase README](../README.md) (linked list with Box, tree with Rc and Weak,
graph with shared ownership, cache with Arc and Mutex).

In the exercise skeleton, `Drop::drop` bodies are left **empty** instead of
`todo!()`: a panic inside `drop` while another panic is unwinding aborts the
whole test process.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

Phase 2 complete. Continue with
[Phase 3 · 11 Advanced Lifetimes](../../03-advanced-rust/11-lifetimes-advanced/).
