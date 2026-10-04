# 02 · Basic Syntax

## Overview

Variables, types, functions, `if`, loops and `match`. Every language has them;
what is different in Rust is that almost everything is an **expression** — an
`if`, a `match`, a `loop`, a `{ block }` all produce values — and that the
compiler insists every case is handled.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Variables | Immutable by default; `mut` vs shadowing; `const` |
| 2 | Data types | Integer sizes, `char` is 4 bytes, tuples, arrays, overflow |
| 3 | Functions | The last expression is the return value; passing functions |
| 4 | If/else | `if` is an expression, so both arms need one type |
| 5 | Loops | `loop` returns a value with `break`, loop labels |
| 6 | Match | Exhaustiveness, ranges, guards, `Option` |
| 7 | Calculator | `?` on `Option`, parsing, an input loop |
| 8 | FizzBuzz | Three styles that must agree |
| 9 | Temperature | Combining all of it |
| Bonus | Primes | Trial division vs the Sieve of Eratosthenes |

## Key Concepts

### Shadowing is not mutation

```rust
let spaces = "   ";          // &str
let spaces = spaces.len();   // usize: a NEW variable with the same name

let mut count = "   ";
count = count.len();         // error[E0308]: mut can't change the type
```

Use `mut` when a value changes over time. Use shadowing when you *transform* a
value and the old form is no longer useful (`let input = input.trim();`).

### Everything is an expression

```rust
let grade = if score >= 90 { 'A' } else { 'B' };      // if returns a value
let first = loop { if ok() { break 7; } };             // so does loop
let label = match n { 0 => "zero", _ => "other" };     // and match
```

That is why both `if` arms must have the same type, and why `return` is rare in
idiomatic Rust.

### `match` must be exhaustive

```rust
match coin {
    Coin::Penny => 1,
    Coin::Nickel => 5,
    Coin::Dime => 10,
    // forgot Quarter -> error[E0004]: non-exhaustive patterns
}
```

Add a variant to an enum and every `match` on it stops compiling until you deal
with it. That is a feature: the compiler finds every place you need to update.

### Integer overflow depends on the build

`127i8 + 1` panics in a debug build and silently wraps to `-128` in release.
When overflow is possible, choose explicitly: `wrapping_add`, `checked_add`
(returns `Option`), `saturating_add`, or `overflowing_add`.

### `?` works on `Option` too

```rust
fn parse_operation(input: &str) -> Option<(f64, char, f64)> {
    let mut parts = input.split_whitespace();
    let a: f64 = parts.next()?.parse().ok()?;   // any None returns early
    ...
}
```

## Common Pitfalls

1. **Semicolon on the last line** of a function — it now returns `()`
2. **`if` arms of different types** — they must match
3. **`while` with no progress** — an infinite loop; prefer `for` over ranges
4. **Overflow you didn't think about** — debug panics, release wraps
5. **`for i in 0..n` vs `0..=n`** — `..` excludes the end, `..=` includes it

## Running This Module

```bash
cargo run  -p m02-basic-syntax -- 5                      # your code, exercise 5
cargo test -p m02-basic-syntax-tests --features mine     # test your code
cargo run  -p m02-basic-syntax-solution -- all           # the reference solution
cargo test -p m02-basic-syntax-tests                     # 29 tests against the solution
```

## Notes on `exercises.md`

- **Exercise 5** asks for `print_fibonacci` and `countdown` that print. The
  solution writes `fibonacci(n) -> Vec<u64>` and `countdown(n) -> Vec<String>`
  and prints in `run()`. Same output, but now it can be tested.
- **Exercise 2 bonus** (overflow in debug vs release): try both yourself —
  `cargo run -p m02-basic-syntax-solution -- 2` and add `--release`. The
  solution shows the four explicit alternatives.
- Answers to the written questions are in [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[03 · Ownership & Borrowing](../03-ownership-borrowing/)
