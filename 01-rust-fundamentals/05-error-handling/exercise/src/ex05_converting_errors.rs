//! Exercise 5: Converting Between Error Types.
//!
//! One function, two failure types (io::Error, ParseIntError). `?` needs a
//! single return type that both convert into. Two ways:

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::num::ParseIntError;
use std::path::Path;

/// Task 1: `Box<dyn Error>` -- any error converts into it. Easy, but callers
/// can only print the error or try to downcast it.
pub fn read_number_boxed(path: &Path) -> Result<i32, Box<dyn Error>> {
    todo!("Exercise 5")
}

/// Task 2: an enum that wraps each source error. Callers can match on it.
#[derive(Debug)]
pub enum MyError {
    Io(io::Error),
    Parse(ParseIntError),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 5")
    }
}

/// `source()` exposes the wrapped error, so tools that walk error chains
/// (anyhow's `{:#}`, logging libraries) can show the underlying cause.
impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        todo!("Exercise 5")
    }
}

/// These two impls are what make `?` work: it calls `From::from(err)`.
impl From<io::Error> for MyError {
    fn from(error: io::Error) -> Self {
        todo!("Exercise 5")
    }
}

impl From<ParseIntError> for MyError {
    fn from(error: ParseIntError) -> Self {
        todo!("Exercise 5")
    }
}

pub fn read_number(path: &Path) -> Result<i32, MyError> {
    todo!("Exercise 5")
}

pub fn run() {
    todo!("Exercise 5")
}
