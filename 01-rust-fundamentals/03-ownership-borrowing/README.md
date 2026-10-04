# 03 · Ownership & Borrowing

## Overview

This is the module that makes Rust Rust. Every other language you've used
answers "who frees this memory?" with either *you* (C, C++) or *a garbage
collector* (Java, C#, Go, Python). Rust answers: **the owner, at the end of its
scope — and the compiler proves there is exactly one.**

Expect to fight the borrow checker this week. Every fight is the compiler
showing you a bug that, in another language, would have shipped.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Ownership | Move semantics; three ways to fix "use of moved value" |
| 2 | References | `&T` and `&mut T`; a borrow ends at its last use |
| 3 | `String` vs `&str` | Slices borrow from the original; reverse by `char` |
| 4 | Dangling references | Why `fn dangle() -> &String` can't compile |
| 5 | Clone vs Copy | Which types are Copy, and why `String` can't be |
| 6 | Borrow rules | One writer XOR many readers, in a real struct |
| 7 | Text buffer | `&self` vs `&mut self` as an API contract |
| 8 | Collections | You can't move out of a `Vec` index; `retain`, `extract_if` |
| Bonus | `SimpleRc` | Reference counting by hand, with `unsafe` |

## Key Concepts

### The three rules

1. Each value has exactly **one owner**.
2. Assigning or passing a value **moves** ownership (unless the type is `Copy`).
3. When the owner goes out of scope, the value is **dropped**.

```rust
let s1 = String::from("hello");
let s2 = s1;              // move: s1 is now invalid
println!("{s1}");         // error[E0382]: borrow of moved value: `s1`
```

If both were valid, both would free the same heap buffer — a double free.

### The borrowing rule

At any moment: **one `&mut T`, or any number of `&T`. Never both.**

```rust
let r1 = &s;
let r2 = &s;
println!("{r1} {r2}");    // last use of r1 and r2: their borrows end HERE
let r3 = &mut s;          // fine
```

A borrow lasts until its **last use**, not until the end of the block
(non-lexical lifetimes). Most "cannot borrow as mutable" errors are fixed by
moving a line, not by cloning.

### Why the rule exists

```rust
let first = &v[0];        // a pointer into v's heap buffer
v.push(4);                // may reallocate the buffer...
println!("{first}");      // ...and first now points at freed memory
```

Rust rejects this at compile time. C++ lets it through and you find out in
production.

### Take `&str`, return `String`

| Parameter type | Accepts | Use when |
|---|---|---|
| `&str` | literals, `&String`, slices | you only read the text (almost always) |
| `&mut String` | a mutable `String` | you append or modify in place |
| `String` | an owned `String` | you need to store it or give it away |

### Copy vs Clone

`Copy` is an implicit, bitwise copy — only for types where that is a complete
copy: numbers, `bool`, `char`, shared references, and tuples/arrays/structs
made only of those. `Clone` is an explicit `.clone()` call that may allocate.
Anything owning heap memory can be `Clone` but never `Copy`.

## Common Pitfalls

1. **Reaching for `.clone()` first** — usually the fix is a borrow, or moving a line
2. **`&String` / `&Vec<T>` parameters** — take `&str` / `&[T]` instead
3. **Holding a reference across a mutation** — clone the value out, or restructure
4. **Byte-indexing strings** — `&s[0..1]` panics on multi-byte characters
5. **`for x in v` when you need `v` afterwards** — that consumes it; use `for x in &v`

## Running This Module

```bash
cargo run  -p m03-ownership-borrowing -- 3                     # your code, exercise 3
cargo test -p m03-ownership-borrowing-tests --features mine    # test your code
cargo run  -p m03-ownership-borrowing-solution -- all          # the reference solution
cargo test -p m03-ownership-borrowing-tests                    # 21 tests against the solution
```

## Notes on `exercises.md`

- **Exercise 7** asserts `buffer.search("l") == vec![9, 13]` for
  `"Say: Hello World"`. That's a typo: `l` is at byte offsets **7, 8 and 14**.
  The solution and tests use the correct values.
- **Exercise 8, Task 3** suggests `drain_filter`. It was never stabilised; it
  shipped in Rust 1.87 as `Vec::extract_if(.., pred)`. Both `retain` and
  `extract_if` are in the solution.
- **Exercise 5** uses `3.14` as an example `f64`. Clippy rejects that literal
  (`approx_constant`, deny by default) because it looks like a sloppy `PI`; the
  solution uses `2.5`.
- **Exercise 2** uses `fn calculate_length(s: &String)`. It works, but `&str` is
  idiomatic; clippy's `ptr_arg` lint says the same.
- **Bonus**: the exercise gives `SimpleRc` an inherent `fn clone(&self)`. The
  solution implements the `Clone` trait instead, and uses `NonNull` rather than
  `*mut` so the pointer can never be null. Each `unsafe` block carries a
  `SAFETY:` comment explaining why it's sound.
- Answers to the written questions: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[04 · Structs & Enums](../04-structs-enums/)
