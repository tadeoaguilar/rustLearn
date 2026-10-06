//! Exercise 2: `logview` -- filtering and summarising log files.
//!
//! Log lines look like
//!
//! ```text
//! 2026-10-05T12:00:03Z WARN  app::db: slow query took 812 ms
//! ```
//!
//! -- an RFC 3339 UTC timestamp, a level, a target, a message. Fixed-format
//! UTC timestamps sort correctly as strings, so time ranges need no date
//! library. Subcommands (`filter`, `stats`, `tail`) share one parser.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for Level {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, ParseError> {
        todo!("Exercise 2")
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Exercise 2")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogLine {
    pub timestamp: String,
    pub level: Level,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("not a timestamp: {0:?}")]
    Timestamp(String),
    #[error("unknown level {0:?}")]
    Level(String),
    #[error("missing field")]
    Missing,
}

/// `YYYY-MM-DDTHH:MM:SSZ`, digits where digits go.
pub fn is_timestamp(s: &str) -> bool {
    todo!("Exercise 2")
}

/// Parse one line: `<timestamp> <LEVEL> <target>: <message>` (any amount
/// of whitespace between the first three fields).
pub fn parse_line(line: &str) -> Result<LogLine, ParseError> {
    todo!("Exercise 2")
}

/// Which lines to keep.
#[derive(Args, Debug, Clone, Default, PartialEq, Eq)]
pub struct LineFilter {
    /// Minimum level.
    // TODO Exercise 2: #[arg(...)]
    pub level: Option<Level>,
    /// Target prefix (`app::db` matches `app::db::pool`).
    // TODO Exercise 2: #[arg(...)]
    pub target: Option<String>,
    /// From this timestamp (inclusive).
    // TODO Exercise 2: #[arg(...)]
    pub since: Option<String>,
    /// Before this timestamp (exclusive).
    // TODO Exercise 2: #[arg(...)]
    pub until: Option<String>,
    /// Message contains this text (case-insensitive).
    // TODO Exercise 2: #[arg(...)]
    pub contains: Option<String>,
}

impl LineFilter {
    pub fn matches(&self, line: &LogLine) -> bool {
        todo!("Exercise 2")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stats {
    pub by_level: BTreeMap<Level, usize>,
    pub by_target: BTreeMap<String, usize>,
    pub malformed: usize,
    pub first: Option<String>,
    pub last: Option<String>,
}

/// Count lines per level and target; malformed lines are counted, not fatal.
pub fn stats<'a>(lines: impl IntoIterator<Item = &'a str>) -> Stats {
    todo!("Exercise 2")
}

#[derive(Parser, Debug)]
#[command(name = "logview", about = "Filter and summarise log files")]
pub struct LogviewCli {
    #[command(subcommand)]
    pub command: LogCommand,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum LogCommand {
    /// Print the lines that match.
    Filter {
        #[command(flatten)]
        filter: LineFilter,
        /// One JSON object per line.
        // TODO Exercise 2: #[arg(...)]
        json: bool,
        /// The log file (stdin if omitted).
        file: Option<PathBuf>,
    },
    /// Count lines by level and target.
    Stats { file: Option<PathBuf> },
    /// The last N lines that match the level filter.
    Tail {
        // TODO Exercise 2: #[arg(...)]
        lines: usize,
        // TODO Exercise 2: #[arg(...)]
        level: Option<Level>,
        file: Option<PathBuf>,
    },
}

impl LogCommand {
    pub fn file(&self) -> Option<&PathBuf> {
        match self {
            LogCommand::Filter { file, .. }
            | LogCommand::Stats { file }
            | LogCommand::Tail { file, .. } => file.as_ref(),
        }
    }
}

/// Run a command over `input`, writing to `out`.
pub fn run(command: &LogCommand, input: impl BufRead, out: &mut impl Write) -> io::Result<()> {
    todo!("Exercise 2")
}
