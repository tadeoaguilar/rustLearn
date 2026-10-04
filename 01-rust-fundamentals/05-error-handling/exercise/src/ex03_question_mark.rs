//! Exercise 3: The ? Operator.
//!
//! `expr?` means: if `expr` is `Err(e)`, return `Err(From::from(e))` from the
//! current function right now; otherwise unwrap the `Ok` value. The four
//! versions below behave identically.

use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;
use std::path::Path;

/// v1: manual propagation.
pub fn read_username_v1(path: &Path) -> Result<String, io::Error> {
    todo!("Exercise 3")
}

/// v2: `?` replaces each match.
pub fn read_username_v2(path: &Path) -> Result<String, io::Error> {
    todo!("Exercise 3")
}

/// v3: chained.
pub fn read_username_v3(path: &Path) -> Result<String, io::Error> {
    todo!("Exercise 3")
}

/// v4: the standard library already has this function.
pub fn read_username_v4(path: &Path) -> Result<String, io::Error> {
    todo!("Exercise 3")
}

/// Task 2.
pub fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {
    todo!("Exercise 3")
}

pub fn sum_from_strings(s1: &str, s2: &str) -> Result<i32, ParseIntError> {
    todo!("Exercise 3")
}

pub fn run() {
    todo!("Exercise 3")
}
