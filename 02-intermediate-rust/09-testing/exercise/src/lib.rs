//! Module 09 -- Testing. YOUR WORKSPACE.
//!
//! This library compiles, runs and looks fine. It contains SIX bugs.
//! Your job (see ../exercises.md) is to write the tests that find them:
//!
//! | Exercise | Write                                   | Where                         |
//! |----------|-----------------------------------------|-------------------------------|
//! | 1        | unit tests for `stats`                  | `mod tests` in stats.rs       |
//! | 2        | error and panic tests for `account`     | `mod tests` in account.rs     |
//! | 3        | integration tests + shared helpers      | tests/bank.rs, tests/common/mod.rs |
//! | 4        | doc tests for `slugify`, `to_roman`     | the `///` comments            |
//! | 5        | fakes and mockall mocks for `weather`   | `mod tests` in weather.rs     |
//! | 6        | proptest properties for `roman`, `slug` | tests/properties.rs           |
//! | 7        | criterion benchmarks for `words`        | benches/words.rs              |
//!
//!     cargo test -p m09-testing                          # YOUR tests
//!     cargo test -p m09-testing-tests --features mine    # the answer key
//!
//! When one of your tests fails because of a bug, fix the bug and keep the
//! test.

// The empty `mod tests` blocks and the bowling skeleton would warn until you
// fill them in. Remove this line once you have.
#![allow(unused)]

pub mod account;
pub mod bowling;
pub mod roman;
pub mod slug;
pub mod stats;
pub mod weather;
pub mod words;
