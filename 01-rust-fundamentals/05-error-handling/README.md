# 05 · Error Handling

## Overview

Rust has no exceptions. A function that can fail says so in its return type —
`Result<T, E>` — and the caller can't get at the `T` without deciding what to
do about the `E`. The `?` operator keeps that from becoming tedious, and a
good error type keeps it from becoming useless.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | `panic!` | Panics are for bugs, not for bad input |
| 2 | `Result` | String errors vs enum errors; `checked_div` |
| 3 | `?` | Four equivalent ways to read a file |
| 4 | Custom errors | `Display` + `Error`; struct vs enum errors |
| 5 | Converting errors | `Box<dyn Error>` vs an enum with `From` impls |
| 6 | Option ↔ Result | `ok_or` vs `ok_or_else`; `transpose` |
| 7 | Recovery | Retry with `FnMut`, backoff, fallbacks |
| 8 | Config parser | Line numbers in errors; parsing separated from I/O |
| 9 | `thiserror` | The derive that writes ex05 for you; `anyhow` for apps |
| Bonus | Combinators | Collecting *all* errors without `unwrap` |

## Key Concepts

### Panic or Result?

| Situation | Use |
|---|---|
| A bug: an invariant you rely on is broken | `panic!`, `unreachable!`, `expect("why")` |
| Bad input, missing file, network down | `Result` |
| Tests, examples, prototypes | `unwrap()` / `expect()` is fine |

The test: *could this happen even if every line of code were correct?* If so,
it's not a bug — return a `Result`.

### What `?` actually does

```rust
let n: i32 = s.parse()?;
// is shorthand for
let n: i32 = match s.parse() {
    Ok(v) => v,
    Err(e) => return Err(From::from(e)),
};
```

The `From::from` is the important part: it's why one function can use `?` on
an `io::Error` and a `ParseIntError` and return a single `MyError`.

### Choosing an error type

| | Good for | Caller can |
|---|---|---|
| `String` | quick scripts | print it |
| `Box<dyn Error>` | small apps | print it, try to downcast |
| your own enum (+ `thiserror`) | **libraries** | `match` on every case |
| `anyhow::Error` | **applications** | print it with context: `{:#}` |

### Collect errors without `unwrap`

```rust
match (validate_username(u), validate_email(e), validate_age(a)) {
    (Ok(username), Ok(email), Ok(age)) => Ok(User { username, email, age }),
    (u, e, a) => Err([u.err(), e.err(), a.err()].into_iter().flatten().collect()),
}
```

No `unwrap()` means no way for a later edit to make one panic.

## Common Pitfalls

1. **`unwrap()` in library code** — the caller loses the choice
2. **`ok_or(format!(...))`** — builds the error even on success; use `ok_or_else`
3. **`Box<dyn Error>` in a library API** — callers can't match on it
4. **Losing the cause** — wrap errors (`Io(io::Error)`) instead of `.to_string()`-ing them
5. **Hard-coded file paths** — take a `&Path` so the code can be tested

## Running This Module

```bash
cargo run  -p m05-error-handling -- 8                      # your code, exercise 8
cargo test -p m05-error-handling-tests --features mine     # test your code
cargo run  -p m05-error-handling-solution -- all           # the reference solution
cargo test -p m05-error-handling-tests                     # 30 tests against the solution
```

## Notes on `exercises.md`

- **File paths**: the exercises hard-code `"username.txt"` and `"config.txt"`.
  The solution takes a path parameter instead; the tests write throwaway files
  with the `tempfile` crate, and `run()` uses the OS temp directory.
- **Exercise 7**: `retry(op, max_retries)` stops when `attempts >= max_retries`,
  so `max_retries` is really the maximum number of **attempts**. The solution
  adds `retry_with_delay` so tests don't sleep, plus an exponential backoff.
- **Exercise 6**: "transpose (requires iterator)" — it doesn't; `transpose` is a
  method on `Option<Result<..>>` and `Result<Option<..>>` directly.
- **Exercise 8**: a missing key is reported as `InvalidValue { value: "" }`.
  The solution adds a `MissingKey` variant so callers can tell "absent" from
  "malformed". It also separates parsing (`from_reader`) from file I/O.
- **Exercise 9**: `thiserror = "1.0"` — the workspace uses 2.x; the derive syntax
  used here is identical. `anyhow` is added alongside for comparison.
- **Exercise 1**: `catch_unwind` in `run()` exists only so the demo can show
  three panics in one run. It is not an error-handling mechanism.
- Answers to the written questions: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

Phase 1 complete. Continue with
[Phase 2 · 06 Traits & Generics](../../02-intermediate-rust/06-traits-generics/).
