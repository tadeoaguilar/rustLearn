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
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {e}"),
            ConfigError::InvalidFormat { line_num, line } => {
                write!(f, "Invalid format at line {line_num}: {line}")
            }
            ConfigError::MissingKey { key } => write!(f, "Missing key '{key}'"),
            ConfigError::InvalidValue { key, value, reason } => {
                write!(f, "Invalid value for '{key}' = '{value}': {reason}")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::Io(error)
    }
}

#[derive(Debug, Default)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let file = File::open(path)?;
        Self::from_reader(BufReader::new(file))
    }

    /// Accepts anything readable line by line -- a file, a socket, or a
    /// `&[u8]` in a test.
    pub fn from_reader(reader: impl BufRead) -> Result<Self, ConfigError> {
        let mut values = HashMap::new();
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // split_once is the modern spelling of splitn(2, '=') + collect.
            let Some((key, value)) = line.split_once('=') else {
                return Err(ConfigError::InvalidFormat {
                    line_num: index + 1,
                    line: line.to_string(),
                });
            };
            let key = key.trim();
            if key.is_empty() {
                return Err(ConfigError::InvalidFormat {
                    line_num: index + 1,
                    line: line.to_string(),
                });
            }
            values.insert(key.to_string(), value.trim().to_string());
        }
        Ok(Config { values })
    }

    /// Convenience for tests and examples.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        Self::from_reader(text.as_bytes())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or(default).to_string()
    }

    /// The exercise reports a missing key as InvalidValue with an empty value.
    /// A dedicated `MissingKey` variant lets callers tell "you forgot it" from
    /// "you wrote it wrong" -- e.g. to fall back to a default only for the first.
    fn require(&self, key: &str) -> Result<&str, ConfigError> {
        self.get(key).ok_or_else(|| ConfigError::MissingKey {
            key: key.to_string(),
        })
    }

    pub fn get_int(&self, key: &str) -> Result<i32, ConfigError> {
        let value = self.require(key)?;
        value
            .parse()
            .map_err(|e: std::num::ParseIntError| ConfigError::InvalidValue {
                key: key.to_string(),
                value: value.to_string(),
                reason: format!("not a valid integer ({e})"),
            })
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        let value = self.require(key)?;
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(ConfigError::InvalidValue {
                key: key.to_string(),
                value: value.to_string(),
                reason: "not a valid boolean".to_string(),
            }),
        }
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
    let config = Config::from_file(path)?;
    let port = config.get_int("port")?;
    let host = config.get_or("host", "0.0.0.0");
    let debug = config.get_bool("debug")?;
    Ok(format!(
        "Server starting on {host}:{port}\nDebug mode: {debug}"
    ))
}

pub fn run() {
    let path = std::env::temp_dir().join("rustlearn-m05-config.txt");
    std::fs::write(&path, SAMPLE).unwrap();
    match start_server(&path) {
        Ok(msg) => println!("{msg}"),
        Err(e) => println!("Error: {e}"),
    }
    let _ = std::fs::remove_file(&path);

    for bad in [
        "port = eighty",
        "debug = maybe\nport = 1",
        "just some words",
        "host = x",
    ] {
        let result = Config::parse(bad).and_then(|c| {
            c.get_int("port")?;
            c.get_bool("debug")
        });
        println!(
            "{bad:?} -> {}",
            result.map_or_else(|e| e.to_string(), |v| v.to_string())
        );
    }
    println!(
        "missing file -> {}",
        start_server("/no/such/config.txt").unwrap_err()
    );
}
