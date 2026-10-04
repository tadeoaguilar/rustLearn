# Exercises: Testing

In every other module you write code and the tests are given. Here it's the
other way round: the code is given, and **you write the tests**.

`exercise/src/` contains a small library that compiles, runs, and looks fine:

| Module | What it does |
|---|---|
| `stats` | mean, median, mode, standard deviation |
| `account` | bank accounts: deposit, withdraw, transfer, freeze |
| `slug` | turns titles into URL slugs |
| `roman` | Roman numerals, both directions |
| `weather` | a clothing advisor that calls a weather API |
| `words` | word counting (two implementations, for benchmarking) |

**It contains six bugs.** None of them shows up in `cargo run`. Your tests
should find all six. When a test fails, fix the bug, and keep the test — it is
now a regression test.

There is an answer key: `cargo test -p m09-testing-tests --features mine` runs
acceptance tests against your crate. Try to find the bugs with your own tests
before you look at what it reports.

---

## Exercise 1: Unit Tests

**Difficulty**: Easy
**Time**: 30 minutes

**Learning Objectives**:
- Write `#[test]` functions in a `#[cfg(test)] mod tests` block
- Use `assert!`, `assert_eq!`, `assert_ne!` with custom messages
- Think in edge cases: empty input, one element, even/odd length, duplicates

**Task**: Test every function in `stats.rs`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_of_simple_values() {
        assert_eq!(mean(&[1.0, 2.0, 3.0]), Some(2.0));
    }

    #[test]
    fn median_of_even_length_averages_the_middle_two() {
        // what should median(&[1.0, 2.0, 3.0, 4.0]) be?
    }

    // empty input? one value? unsorted input? negative values?
}
```

**Run**:
```bash
cargo test -p m09-testing stats
```

**Hints**:
- A test name should say what behaviour it checks: `median_of_even_length_averages_the_middle_two`
- Floats: compare with a tolerance, `(a - b).abs() < 1e-9`, not `==`, unless the value is exact
- There are **two** bugs in `stats.rs`

---

## Exercise 2: Testing Errors and Panics

**Difficulty**: Medium
**Time**: 35 minutes

**Learning Objectives**:
- Test that code fails *correctly*, not just that it succeeds
- Use `#[should_panic(expected = "...")]`
- Write tests that return `Result` and use `?`

**Task**: Test `account.rs`.

```rust
#[test]
#[should_panic(expected = "owner name must not be empty")]
fn new_rejects_empty_owner() {
    Account::new("", 0);
}

#[test]
fn withdraw_more_than_balance_is_an_error() {
    let mut acc = Account::new("Ann", 100);
    assert_eq!(acc.withdraw(150), Err(AccountError::InsufficientFunds { balance: 100, requested: 150 }));
    assert_eq!(acc.balance(), 100, "a failed withdrawal must not change the balance");
}

#[test]
fn deposit_then_withdraw() -> Result<(), AccountError> {
    let mut acc = Account::new("Ann", 0);
    acc.deposit(50)?;
    acc.withdraw(20)?;
    assert_eq!(acc.balance(), 30);
    Ok(())
}
```

**Think about**:
- What should happen to *both* balances when a transfer fails halfway?
- What can a frozen account still do?

**Hints**:
- `matches!(result, Err(AccountError::Frozen))` checks a variant without caring about fields
- There is **one** bug in `account.rs`. It loses money.

---

## Exercise 3: Integration Tests and Test Organisation

**Difficulty**: Medium
**Time**: 30 minutes

**Learning Objectives**:
- Tell unit tests (inside `src/`, can see private items) from integration tests
  (in `tests/`, see only the public API)
- Share helpers between integration test files
- Test private functions from unit tests

**Tasks**:
1. Create `exercise/tests/bank.rs`: an integration test that opens three
   accounts, makes several transfers and checks that **the total amount of
   money never changes**.
2. Put shared setup in `exercise/tests/common/mod.rs` (not
   `tests/common.rs` — that would be compiled as a test file of its own):
   ```rust
   // tests/common/mod.rs
   pub fn three_accounts() -> Vec<Account> { ... }
   ```
   ```rust
   // tests/bank.rs
   mod common;
   ```
3. `slug.rs` has a private helper, `is_separator`. Test it from a unit test in
   the same file — integration tests can't see it.

**Run**:
```bash
cargo test -p m09-testing --test bank      # one integration test file
cargo test -p m09-testing --lib            # unit tests only
```

---

## Exercise 4: Documentation Tests

**Difficulty**: Easy
**Time**: 25 minutes

**Learning Objectives**:
- Write examples in doc comments that `cargo test` compiles and runs
- Use hidden lines (`# `), `?` in examples, `should_panic`, `no_run`, `compile_fail`

**Task**: Document `slug::slugify` and `roman::to_roman` with examples:

```rust
/// Turns a title into a URL slug.
///
/// ```
/// use m09_testing::slug::slugify;
/// assert_eq!(slugify("Hello, World!"), "hello-world");
/// ```
pub fn slugify(title: &str) -> String { ... }
```

Add at least one example that would have caught the bug in `slugify`
(there is **one**).

**Note**: the exercise crate ships with `doctest = false` in its `Cargo.toml`
(like every other exercise crate). **Delete those two lines** for this module —
doc tests are what you're practising.

```bash
cargo test -p m09-testing --doc
```

---

## Exercise 5: Mocking

**Difficulty**: Hard
**Time**: 45 minutes

**Learning Objectives**:
- Design for testability: depend on a trait, not a concrete client
- Write a hand-rolled fake
- Generate mocks with `mockall`

**Background**: `weather::Advisor` decides what to wear. It calls
`WeatherApi::temperature_c(city)`. In production that's an HTTP call; in tests
it must not be.

**Tasks**:
1. Write a hand-rolled fake:
   ```rust
   struct FixedWeather(Result<f64, WeatherError>);
   impl WeatherApi for FixedWeather { ... }
   ```
2. Use `mockall` to check that the advisor asks for the right city and calls
   the API **exactly once**:
   ```rust
   #[cfg_attr(test, mockall::automock)]
   pub trait WeatherApi { ... }

   let mut api = MockWeatherApi::new();
   api.expect_temperature_c()
       .with(mockall::predicate::eq("Oslo"))
       .times(1)
       .returning(|_| Ok(-5.0));
   ```
3. Test every temperature boundary (what does the advisor say at exactly 10°C?)
   and the error path.

**Hints**:
- There is **one** bug in `weather.rs`, at a boundary.
- `mockall` is already a dev-dependency of the exercise crate.

---

## Exercise 6: Property-Based Testing

**Difficulty**: Hard
**Time**: 45 minutes

**Learning Objectives**:
- State properties that hold for *all* inputs instead of picking examples
- Use `proptest` and read its shrunk counterexamples

**Task**: In `exercise/tests/properties.rs`:

```rust
use m09_testing::roman::{from_roman, to_roman};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip(n in 1u32..=3999) {
        let numeral = to_roman(n).unwrap();
        prop_assert_eq!(from_roman(&numeral), Ok(n));
    }
}
```

The roundtrip passes. Does that mean `to_roman` is correct? Write a second
property: in a *canonical* Roman numeral, no symbol appears four times in a row.

Then add properties for `slugify`:
- the output contains only `a-z`, `0-9` and `-`
- it never starts or ends with `-`, and never contains `--`
- slugifying a slug changes nothing (idempotence)

**Hints**:
- When a property fails, proptest *shrinks* the input to the smallest failing case — read it
- There is **one** bug in `roman.rs` that the roundtrip property can't see

---

## Exercise 7: Benchmarking

**Difficulty**: Medium
**Time**: 30 minutes

**Learning Objectives**:
- Measure instead of guessing
- Use `criterion` and `std::hint::black_box`

**Task**: `words.rs` has two implementations of the same function:
`count_words_split` (split + HashMap of `String`) and `count_words_borrowed`
(HashMap of `&str`, no allocation per word). Benchmark them on a large text in
`exercise/benches/words.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    let text = "the quick brown fox ".repeat(10_000);
    c.bench_function("split", |b| b.iter(|| count_words_split(black_box(&text))));
    c.bench_function("borrowed", |b| b.iter(|| count_words_borrowed(black_box(&text))));
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

```bash
cargo bench -p m09-testing
```

**Questions**:
- Why `black_box`?
- Why must benchmarks run in release mode, and why does `cargo bench` do that for you?

---

## Bonus Challenge: Coverage and TDD

**Difficulty**: Medium
**Time**: 60 minutes

1. Measure coverage:
   ```bash
   cargo install cargo-llvm-cov
   cargo llvm-cov -p m09-testing --html --open
   ```
   Find a line no test executes and cover it.
2. **TDD kata**: implement `bowling::score(rolls: &[u32]) -> u32` test-first.
   Write one failing test, make it pass with the simplest code, refactor,
   repeat: gutter game (0), all ones (20), one spare, one strike, perfect game (300).

---

## The Six Bugs

Don't read this until you're done.

<details>
<summary>Show</summary>

1. `stats::mean` of an empty slice returns `Some(NaN)` instead of `None`
2. `stats::median` of an even-length slice returns the upper middle value instead of the average of the two middle values
3. `account::transfer` takes money out of the source, then fails to deposit into a frozen target — and doesn't put it back
4. `weather::Advisor::advise` at exactly 25°C returns `Mild`, but the documented rule says `Hot` from 25°C
5. `roman::to_roman` writes 90 as `LXXXX` and 900 as `DCCCC` (missing `XC` and `CM`); `from_roman` happily parses them back, so the roundtrip test passes
6. `slug::slugify("a  b")` produces `a--b`: runs of separators aren't collapsed

</details>

---

## Check Your Understanding

- [ ] Write unit tests next to the code, integration tests in `tests/`
- [ ] Test error paths and panics, not only the happy path
- [ ] Share helpers between integration tests with `tests/common/mod.rs`
- [ ] Write doc examples that are compiled and run
- [ ] Replace a dependency with a fake or a mock through a trait
- [ ] State and test properties with `proptest`
- [ ] Benchmark with `criterion` and interpret the result

---

## Additional Resources

- [Rust Book Chapter 11](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest book](https://proptest-rs.github.io/proptest/)
- [mockall docs](https://docs.rs/mockall/)
- [Criterion.rs user guide](https://bheisler.github.io/criterion.rs/book/)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
