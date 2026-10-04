//! Exercise 6: Option to Result Conversion.

use std::num::ParseIntError;

/// Simulated database lookup.
pub fn get_user_id(username: &str) -> Option<u32> {
    todo!("Exercise 6")
}

/// `ok_or(err)` builds the error *every time*, even on success -- here that
/// means a format! allocation on every call.
#[allow(clippy::or_fun_call)] // exactly what this function demonstrates
pub fn get_user_id_result(username: &str) -> Result<u32, String> {
    todo!("Exercise 6")
}

/// `ok_or_else(|| err)` only builds it when needed. Prefer it whenever the
/// error isn't a constant.
pub fn get_user_id_lazy(username: &str) -> Result<u32, String> {
    todo!("Exercise 6")
}

/// Task 2. `Option<Result<T, E>>` <-> `Result<Option<T>, E>`.
pub fn parse_optional(s: Option<&str>) -> Result<Option<i32>, ParseIntError> {
    todo!("Exercise 6")
}

/// `s.map(str::parse)` is an `Option<Result<i32, _>>`; `transpose` flips it.
/// (The exercise says "requires iterator" -- it doesn't; transpose is a
/// method on Option and Result themselves.)
pub fn parse_optional_v2(s: Option<&str>) -> Result<Option<i32>, ParseIntError> {
    todo!("Exercise 6")
}

pub fn run() {
    todo!("Exercise 6")
}
