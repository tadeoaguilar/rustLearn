//! Module 09 -- Testing. Reference solution.
//!
//! The library with all six bugs fixed, and a test suite of every kind:
//!
//! | Kind                | Where                                   |
//! |---------------------|-----------------------------------------|
//! | unit tests          | `#[cfg(test)] mod tests` in each file   |
//! | integration tests   | `tests/bank.rs` (+ `tests/common/mod.rs`) |
//! | property tests      | `tests/properties.rs`                   |
//! | doc tests           | the examples in these doc comments      |
//! | mocks               | `weather.rs` unit tests (mockall)       |
//! | benchmarks          | `benches/words.rs` (criterion)          |

pub mod account;
pub mod bowling;
pub mod roman;
pub mod slug;
pub mod stats;
pub mod weather;
pub mod words;
