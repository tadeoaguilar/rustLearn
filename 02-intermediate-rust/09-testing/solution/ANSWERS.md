# Answers · 09 Testing

## Where the six bugs are, and which test finds each

| # | Bug | Found by | Test |
|---|---|---|---|
| 1 | `mean(&[])` is `Some(NaN)` | unit test, edge case | `stats::tests::mean_of_nothing_is_none_not_nan` |
| 2 | even-length `median` returns the upper middle | unit test, edge case | `stats::tests::median_of_even_length_averages_the_middle_two` |
| 3 | `transfer` to a frozen account loses the money | error-path test; integration test | `account::tests::failed_transfer_to_frozen_account_changes_nothing`, `tests/bank.rs` |
| 4 | 25°C is `Mild` instead of `Hot` | boundary test with a fake | `weather::tests::every_boundary` |
| 5 | 90 → `LXXXX`, 900 → `DCCCC` | property test (canonical form) | `tests/properties.rs::roman_is_canonical` |
| 6 | `slugify("a  b")` → `a--b` | doc test; property test | the `slugify` doc example; `slug_has_no_stray_hyphens` |

Every fix in `src/` is marked with a `BUG FIXED` comment.

## Exercise 7

**Why `black_box`?**

The optimiser is allowed to compute anything it can prove at compile time, and
to delete code whose result is unused. A benchmark that calls a pure function
with a constant input and ignores the result can be optimised down to nothing,
and you'd measure an empty loop. `std::hint::black_box(x)` returns `x` but
tells the compiler to assume it can't see through it — so the input looks
unknown and the output looks used.

**Why must benchmarks run in release mode?**

A debug build has no optimisations and adds overflow checks; it can be 10–100×
slower, and the slowdown differs between implementations, so even the
*ranking* of two versions can flip. `cargo bench` builds with the `bench`
profile, which inherits from `release`.

**What does the benchmark show?**

On a typical laptop, `count_words_borrowed` is noticeably faster than
`count_words_split` on repetitive text: it never allocates a `String` per word,
only hashes and compares borrowed `&str`s. The cost is a lifetime: the map
borrows from the text and can't outlive it.
