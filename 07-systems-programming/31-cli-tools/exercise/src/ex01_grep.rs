//! Exercise 1: `minigrep` -- a grep clone with clap.
//!
//! A command-line tool is an API: its arguments, its output on stdout, its
//! diagnostics on stderr and its exit code are all things scripts depend on.
//! This one follows grep's conventions: exit 0 if something matched, 1 if
//! nothing did, 2 on an error; file names prefixed when searching several.

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use regex::{Regex, RegexBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum ColorChoice {
    Always,
    #[default]
    Never,
    /// Only when writing to a terminal.
    Auto,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(
    name = "minigrep",
    version,
    about = "Search files for lines matching a pattern"
)]
pub struct GrepArgs {
    /// The regular expression (or fixed string with -F).
    pub pattern: String,
    /// Files or directories (with -r) to search.
    // TODO Exercise 1: #[arg(...)]
    pub paths: Vec<PathBuf>,
    /// Match case-insensitively.
    // TODO Exercise 1: #[arg(...)]
    pub ignore_case: bool,
    /// Prefix each line with its line number.
    // TODO Exercise 1: #[arg(...)]
    pub line_number: bool,
    /// Select the lines that do NOT match.
    // TODO Exercise 1: #[arg(...)]
    pub invert_match: bool,
    /// Print only a count of matching lines per file.
    // TODO Exercise 1: #[arg(...)]
    pub count: bool,
    /// Search directories recursively.
    // TODO Exercise 1: #[arg(...)]
    pub recursive: bool,
    /// Treat the pattern as a literal string.
    // TODO Exercise 1: #[arg(...)]
    pub fixed_strings: bool,
    /// Stop after this many matching lines per file.
    // TODO Exercise 1: #[arg(...)]
    pub max_count: Option<usize>,
    /// Highlight matches.
    // TODO Exercise 1: #[arg(...)]
    pub color: ColorChoice,
}

#[derive(Debug, thiserror::Error)]
pub enum GrepError {
    #[error("invalid pattern: {0}")]
    Pattern(#[from] regex::Error),
    #[error("{0}: is a directory (use -r)")]
    IsADirectory(PathBuf),
    #[error("{0}: {1}")]
    Io(PathBuf, io::Error),
}

/// Compile the pattern, honouring `-F` and `-i`.
pub fn build_regex(args: &GrepArgs) -> Result<Regex, GrepError> {
    todo!("Exercise 1")
}

/// The selected lines of `reader` as `(line number from 1, line)`.
pub fn search_lines(
    reader: impl BufRead,
    re: &Regex,
    invert: bool,
    max_count: Option<usize>,
) -> io::Result<Vec<(usize, String)>> {
    todo!("Exercise 1")
}

/// The files to search: files as given; directories only with `recursive`,
/// walked in sorted order, skipping hidden entries (`.git`, ...).
pub fn collect_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>, GrepError> {
    todo!("Exercise 1")
}

/// `line` with every match wrapped in ANSI bold red.
pub fn highlight(line: &str, re: &Regex) -> String {
    todo!("Exercise 1")
}

/// Run `minigrep`: results on `out`, diagnostics on `err`, the exit code
/// returned (0 matched, 1 no match, 2 error -- an error wins).
pub fn run(args: &GrepArgs, out: &mut impl Write, err: &mut impl Write, is_terminal: bool) -> i32 {
    todo!("Exercise 1")
}
