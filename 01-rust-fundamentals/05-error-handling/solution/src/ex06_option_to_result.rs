//! Exercise 6: Option to Result Conversion.

use std::num::ParseIntError;

/// Simulated database lookup.
pub fn get_user_id(username: &str) -> Option<u32> {
    if username == "alice" { Some(1) } else { None }
}

/// `ok_or(err)` builds the error *every time*, even on success -- here that
/// means a format! allocation on every call.
#[allow(clippy::or_fun_call)] // exactly what this function demonstrates
pub fn get_user_id_result(username: &str) -> Result<u32, String> {
    get_user_id(username).ok_or(format!("User '{username}' not found"))
}

/// `ok_or_else(|| err)` only builds it when needed. Prefer it whenever the
/// error isn't a constant.
pub fn get_user_id_lazy(username: &str) -> Result<u32, String> {
    get_user_id(username).ok_or_else(|| format!("User '{username}' not found"))
}

/// Task 2. `Option<Result<T, E>>` <-> `Result<Option<T>, E>`.
pub fn parse_optional(s: Option<&str>) -> Result<Option<i32>, ParseIntError> {
    match s {
        Some(s) => s.parse().map(Some),
        None => Ok(None),
    }
}

/// `s.map(str::parse)` is an `Option<Result<i32, _>>`; `transpose` flips it.
/// (The exercise says "requires iterator" -- it doesn't; transpose is a
/// method on Option and Result themselves.)
pub fn parse_optional_v2(s: Option<&str>) -> Result<Option<i32>, ParseIntError> {
    s.map(str::parse).transpose()
}

pub fn run() {
    for name in ["alice", "bob"] {
        match get_user_id_result(name) {
            Ok(id) => println!("User ID: {id}"),
            Err(e) => println!("Error: {e}"),
        }
    }
    println!("lazy: {:?}", get_user_id_lazy("bob"));
    for input in [Some("5"), None, Some("x")] {
        println!("parse_optional({input:?}) = {:?}", parse_optional_v2(input));
    }
}
