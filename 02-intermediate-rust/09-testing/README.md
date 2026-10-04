# 09 · Testing

## Overview

Rust's test tooling is built in: `#[test]`, `cargo test`, doc tests and
integration tests need no framework. This module covers those, then the three
external tools every serious Rust project ends up using — **mockall** for
mocks, **proptest** for property-based tests, **criterion** for benchmarks.

It's also the one module where **you write the tests and the code is given**.
The exercise crate is a small library with six bugs that never show up in
`cargo run`. Each exercise practises one kind of test, and each kind finds a
different bug.

## What You'll Learn

| Exercise | Kind of test | Finds |
|---|---|---|
| 1 | Unit tests, edge cases | 2 bugs in `stats` (empty input, even-length median) |
| 2 | Error paths, `should_panic`, `Result` tests | a transfer that loses money |
| 3 | Integration tests, shared helpers | conservation of money across many transfers |
| 4 | Doc tests | a slug with `--` in it |
| 5 | Fakes and mocks | an off-by-one at a temperature boundary |
| 6 | Property-based tests | a Roman numeral bug a roundtrip test can't see |
| 7 | Benchmarks | which word counter is faster, and by how much |
| Bonus | Coverage, TDD | untested lines; the bowling kata |

## Key Concepts

### Where tests live

| Kind | Location | Sees | Run with |
|---|---|---|---|
| Unit | `#[cfg(test)] mod tests` in `src/x.rs` | everything, including private items | `cargo test --lib` |
| Integration | `tests/*.rs` | only the public API | `cargo test --test bank` |
| Doc | ` ``` ` blocks in `///` comments | only the public API | `cargo test --doc` |
| Bench | `benches/*.rs` | only the public API | `cargo bench` |

Shared integration-test helpers go in `tests/common/mod.rs`; a
`tests/common.rs` would be compiled as a test binary of its own.

### Test the failure, not just the success

```rust
assert_eq!(acc.withdraw(150), Err(AccountError::InsufficientFunds { balance: 100, requested: 150 }));
assert_eq!(acc.balance(), 100, "a failed withdrawal must not change the balance");
```

The second line is the one that catches bugs: *what state is left behind* when
something fails? Bug #3 is a transfer that fails halfway and keeps the money.

### Boundaries

Bug #4 is `t > 25.0` where the rule says "25°C and above". Test *at* every
boundary and just below it. Off-by-one errors live there and nowhere else.

### Properties beat examples — but choose them well

```rust
proptest! {
    #[test]
    fn roundtrip(n in 1u32..=3999) {
        prop_assert_eq!(from_roman(&to_roman(n).unwrap()), Ok(n));
    }
}
```

This passes even though `to_roman(90)` returns `"LXXXX"`, because
`from_roman` accepts it. A roundtrip only proves the two functions agree with
each other. A second property — "no symbol four times in a row" — checks the
output against the *specification*, and proptest shrinks the failure to `n = 90`.

### Depend on traits to make code testable

```rust
pub struct Advisor<A: WeatherApi> { api: A }
```

Because `Advisor` takes any `WeatherApi`, tests can pass a three-line fake or a
`mockall` mock that also checks *how* it was called (`.with(eq("Oslo")).times(1)`).

## Common Pitfalls

1. **Comparing floats with `==`** — use a tolerance unless the value is exact
2. **Only testing the happy path** — most bugs live in error handling
3. **`tests/common.rs`** — becomes an empty test binary; use `tests/common/mod.rs`
4. **Benchmarking a debug build** — `cargo bench` uses release; `cargo run` doesn't
5. **Benchmarks without `black_box`** — the optimiser deletes the work you meant to measure
6. **Tests that depend on HashMap order or the current time** — flaky by design

## Running This Module

```bash
cargo test  -p m09-testing                           # YOUR tests against the buggy library
cargo test  -p m09-testing-tests --features mine     # the answer key (6 bugs + bonus fail at first)
cargo test  -p m09-testing-solution                  # the full reference suite: 36 unit, 2 integration, 5 property, 4 doc
cargo test  -p m09-testing-tests                     # 14 acceptance tests against the solution
cargo bench -p m09-testing-solution                  # criterion benchmarks
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of the others and the topics listed in
[the phase README](../README.md). Differences from other modules:

- `exercise/` is **complete, working, buggy code** — not `todo!()` skeletons
  (except the bowling bonus). Its `mod tests` blocks are empty for you to fill.
- `tests/` is an **answer key**: one acceptance test per bug, named
  `bug1_…` to `bug6_…`.
- `solution/` holds the fixed code; every fix is marked `BUG FIXED` with an
  explanation, next to the test that catches it.
- The list of bugs is at the end of `exercises.md`, folded away.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[10 · Smart Pointers](../10-smart-pointers/)
