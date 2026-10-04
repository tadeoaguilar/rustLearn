# 07 · Collections & Iterators

## Overview

Most Rust code that "does something to a list" is an iterator chain:
`filter`, `map`, `fold`, `collect`. They're lazy (nothing runs until a consumer
pulls), they compose, and they compile down to the same loop you'd write by
hand. Closures are what you pass to them, so they're in this module too.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | `Vec` | Dedup with and without order, `split`, a linear-time merge |
| 2 | `HashMap` | The `entry` API, `BTreeMap` for sorted output, an LRU cache |
| 3 | Iterator basics | `iter` / `iter_mut` / `into_iter`, laziness you can watch |
| 4 | Advanced adaptors | `fold`, `scan`, `zip`, `flat_map`, `partition`, `successors` |
| 5 | Custom iterators | Implement `next`, get 75 adaptors; a borrowing iterator |
| 6 | Closures | `Fn` / `FnMut` / `FnOnce`, `move`, returning closures |
| Bonus | Data pipeline | Group, aggregate, sort — deterministically |

## Key Concepts

### Three ways to iterate a collection

| Call | Yields | Collection afterwards |
|---|---|---|
| `v.iter()` / `for x in &v` | `&T` | still usable |
| `v.iter_mut()` / `for x in &mut v` | `&mut T` | still usable, modified |
| `v.into_iter()` / `for x in v` | `T` | gone (moved) |

### Iterators are lazy

```rust
let lazy = v.iter().map(|x| { println!("mapping {x}"); x * 10 });
// nothing printed yet
let first_two: Vec<_> = lazy.take(2).collect();   // prints "mapping 1", "mapping 2"
```

Which is why an infinite range is fine: `(1..).find(|n| n % 15 == 0)`.

### The `entry` API

```rust
*counts.entry(word).or_insert(0) += 1;   // one lookup: insert-or-update
groups.entry(age).or_default().push(name);
```

### HashMap order is random

Iterating a `HashMap` gives a different order each run. When order matters —
printing, comparing in tests, "top N" with ties — use a `BTreeMap`, or sort
with an explicit tie-breaker.

### `Fn`, `FnMut`, `FnOnce`

| Closure body | Implements | Accept it with |
|---|---|---|
| only reads captures | `Fn` (and the other two) | `F: Fn(..)` |
| mutates captures | `FnMut` (and `FnOnce`) | `F: FnMut(..)`, call through `mut f` |
| moves a capture out | `FnOnce` only | `F: FnOnce(..)` |

When *accepting* a closure, ask for the weakest trait you need: `FnOnce` if you
call it once, `FnMut` if many times, `Fn` only if you need shared access (e.g.
calling from several threads).

## Common Pitfalls

1. **`collect()` without a type** — the compiler needs `Vec<_>`, `HashMap<_, _>`, `String`...
2. **`&&x` in `filter`** — filter passes `&Item`, and `iter()` items are already `&T`
3. **Unchecked arithmetic in infinite iterators** — Fibonacci overflows u64 at item 94
4. **`partial_cmp(..).unwrap()` on floats** — panics on NaN; use `f64::total_cmp`
5. **Returning a closure without `move`** — it borrows locals that are about to die

## Running This Module

```bash
cargo run  -p m07-collections-iterators -- 5                      # your code, exercise 5
cargo test -p m07-collections-iterators-tests --features mine     # test your code
cargo run  -p m07-collections-iterators-solution -- all           # the reference solution
cargo test -p m07-collections-iterators-tests                     # 22 tests against the solution
```

## Notes on `exercises.md`

- **Exercise 5**: the given `Fibonacci::next` computes `current + self.next`
  unconditionally. In a debug build that **panics** on the 94th call
  (`attempt to add with overflow`); in release it silently wraps. The solution
  uses `checked_add` and ends the iterator after F93, the largest Fibonacci
  number that fits in a `u64`. A test pins this down.
- **Bonus**: `sort_by(|a, b| b.1.partial_cmp(a.1).unwrap())` panics if any
  revenue is NaN, and ties come out in random order (HashMap). The solution uses
  `total_cmp` and breaks ties by product name.
- **Exercise 1, Task 2** ("split a Vec at a specific value") can be read two
  ways; the solution implements both: `split_on` (every occurrence, like
  `str::split`) and `split_at_value` (first occurrence only).
- **Exercise 2, Task 3** ("cache with get/set/evict") is implemented as a
  capacity-bounded LRU cache with explicit `evict` as well.
- **Bonus, Task 4** leaves the price ranges open: Budget < 5 ≤ Mid < 50 ≤ Premium.

## Next

[08 · Modules & Crates](../08-modules-crates/)
