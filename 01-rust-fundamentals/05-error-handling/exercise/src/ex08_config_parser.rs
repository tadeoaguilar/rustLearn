//! Exercise 8: File Configuration Parser.
//!
//! Parsing is split from file reading: `from_file` opens the file and hands a
//! reader to `from_reader`, which the tests call directly with a string.

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    InvalidFormat {
        line_num: usize,
        line: String,
    },
    MissingKey {
        key: String,
    },
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 8")
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        todo!("Exercise 8")
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        todo!("Exercise 8")
    }
}

#[derive(Debug, Default)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        todo!("Exercise 8")
    }

    /// Accepts anything readable line by line -- a file, a socket, or a
    /// `&[u8]` in a test.
    pub fn from_reader(reader: impl BufRead) -> Result<Self, ConfigError> {
        todo!("Exercise 8")
    }

    /// Convenience for tests and examples.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        todo!("Exercise 8")
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        todo!("Exercise 8")
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        todo!("Exercise 8")
    }

    /// The exercise reports a missing key as InvalidValue with an empty value.
    /// A dedicated `MissingKey` variant lets callers tell "you forgot it" from
    /// "you wrote it wrong" -- e.g. to fall back to a default only for the first.
    fn require(&self, key: &str) -> Result<&str, ConfigError> {
        todo!("Exercise 8")
    }

    pub fn get_int(&self, key: &str) -> Result<i32, ConfigError> {
        todo!("Exercise 8")
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        todo!("Exercise 8")
    }
}

pub const SAMPLE: &str = "\
# This is a comment
port = 8080
host = localhost
debug = true
";

/// The exercise's `main`, returning the error instead of printing it.
pub fn start_server(path: impl AsRef<Path>) -> Result<String, ConfigError> {
    todo!("Exercise 8")
}

pub fn run() {
    todo!("Exercise 8")
}
