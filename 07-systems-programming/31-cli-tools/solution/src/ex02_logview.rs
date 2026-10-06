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
        match s.to_ascii_uppercase().as_str() {
            "TRACE" => Ok(Level::Trace),
            "DEBUG" => Ok(Level::Debug),
            "INFO" => Ok(Level::Info),
            "WARN" | "WARNING" => Ok(Level::Warn),
            "ERROR" => Ok(Level::Error),
            _ => Err(ParseError::Level(s.to_string())),
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        };
        // `pad` (not `write_str`) so `{:<5}` aligns it.
        f.pad(s)
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
    let b = s.as_bytes();
    b.len() == 20
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            10 => *c == b'T',
            13 | 16 => *c == b':',
            19 => *c == b'Z',
            _ => c.is_ascii_digit(),
        })
}

/// Parse one line: `<timestamp> <LEVEL> <target>: <message>` (any amount
/// of whitespace between the first three fields).
pub fn parse_line(line: &str) -> Result<LogLine, ParseError> {
    let (timestamp, rest) = line
        .trim()
        .split_once(char::is_whitespace)
        .ok_or(ParseError::Missing)?;
    if !is_timestamp(timestamp) {
        return Err(ParseError::Timestamp(timestamp.to_string()));
    }
    let (level, rest) = rest
        .trim_start()
        .split_once(char::is_whitespace)
        .ok_or(ParseError::Missing)?;
    let level: Level = level.parse()?;
    let (target, message) = rest
        .trim_start()
        .split_once(": ")
        .ok_or(ParseError::Missing)?;
    Ok(LogLine {
        timestamp: timestamp.to_string(),
        level,
        target: target.to_string(),
        message: message.to_string(),
    })
}

/// Which lines to keep.
#[derive(Args, Debug, Clone, Default, PartialEq, Eq)]
pub struct LineFilter {
    /// Minimum level.
    #[arg(short, long, value_enum)]
    pub level: Option<Level>,
    /// Target prefix (`app::db` matches `app::db::pool`).
    #[arg(short, long)]
    pub target: Option<String>,
    /// From this timestamp (inclusive).
    #[arg(long)]
    pub since: Option<String>,
    /// Before this timestamp (exclusive).
    #[arg(long)]
    pub until: Option<String>,
    /// Message contains this text (case-insensitive).
    #[arg(short = 'g', long = "grep")]
    pub contains: Option<String>,
}

impl LineFilter {
    pub fn matches(&self, line: &LogLine) -> bool {
        self.level.is_none_or(|min| line.level >= min)
            && self.target.as_ref().is_none_or(|t| {
                line.target == *t
                    || line
                        .target
                        .strip_prefix(t.as_str())
                        .is_some_and(|rest| rest.starts_with("::"))
            })
            && self
                .since
                .as_ref()
                .is_none_or(|s| line.timestamp.as_str() >= s.as_str())
            && self
                .until
                .as_ref()
                .is_none_or(|u| line.timestamp.as_str() < u.as_str())
            && self
                .contains
                .as_ref()
                .is_none_or(|c| line.message.to_lowercase().contains(&c.to_lowercase()))
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
    let mut s = Stats::default();
    for raw in lines {
        match parse_line(raw) {
            Ok(line) => {
                *s.by_level.entry(line.level).or_default() += 1;
                *s.by_target.entry(line.target).or_default() += 1;
                if s.first.is_none() {
                    s.first = Some(line.timestamp.clone());
                }
                s.last = Some(line.timestamp);
            }
            Err(_) if raw.trim().is_empty() => {}
            Err(_) => s.malformed += 1,
        }
    }
    s
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
        #[arg(long)]
        json: bool,
        /// The log file (stdin if omitted).
        file: Option<PathBuf>,
    },
    /// Count lines by level and target.
    Stats { file: Option<PathBuf> },
    /// The last N lines that match the level filter.
    Tail {
        #[arg(short = 'n', default_value_t = 10)]
        lines: usize,
        #[arg(short, long, value_enum)]
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
    match command {
        LogCommand::Filter { filter, json, .. } => {
            for raw in input.lines() {
                let raw = raw?;
                if let Ok(line) = parse_line(&raw) {
                    if filter.matches(&line) {
                        if *json {
                            writeln!(
                                out,
                                "{}",
                                serde_json::to_string(&line).expect("serializable")
                            )?;
                        } else {
                            writeln!(out, "{raw}")?;
                        }
                    }
                }
            }
        }
        LogCommand::Stats { .. } => {
            let lines: Vec<String> = input.lines().collect::<io::Result<_>>()?;
            let s = stats(lines.iter().map(String::as_str));
            if let (Some(first), Some(last)) = (&s.first, &s.last) {
                writeln!(out, "from {first} to {last}")?;
            }
            for (level, n) in &s.by_level {
                writeln!(out, "{level:<5} {n}")?;
            }
            for (target, n) in &s.by_target {
                writeln!(out, "  {target}: {n}")?;
            }
            if s.malformed > 0 {
                writeln!(out, "malformed: {}", s.malformed)?;
            }
        }
        LogCommand::Tail { lines, level, .. } => {
            let filter = LineFilter {
                level: *level,
                ..LineFilter::default()
            };
            let mut last: VecDeque<String> = VecDeque::with_capacity(*lines);
            for raw in input.lines() {
                let raw = raw?;
                if parse_line(&raw).is_ok_and(|l| filter.matches(&l)) {
                    if last.len() == *lines {
                        last.pop_front();
                    }
                    if *lines > 0 {
                        last.push_back(raw);
                    }
                }
            }
            for raw in last {
                writeln!(out, "{raw}")?;
            }
        }
    }
    Ok(())
}
