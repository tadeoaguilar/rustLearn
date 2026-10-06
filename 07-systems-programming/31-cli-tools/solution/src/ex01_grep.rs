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
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
    /// Match case-insensitively.
    #[arg(short, long)]
    pub ignore_case: bool,
    /// Prefix each line with its line number.
    #[arg(short = 'n', long)]
    pub line_number: bool,
    /// Select the lines that do NOT match.
    #[arg(short = 'v', long)]
    pub invert_match: bool,
    /// Print only a count of matching lines per file.
    #[arg(short, long)]
    pub count: bool,
    /// Search directories recursively.
    #[arg(short, long)]
    pub recursive: bool,
    /// Treat the pattern as a literal string.
    #[arg(short = 'F', long)]
    pub fixed_strings: bool,
    /// Stop after this many matching lines per file.
    #[arg(short = 'm', long, value_name = "NUM")]
    pub max_count: Option<usize>,
    /// Highlight matches.
    #[arg(long, value_enum, default_value_t = ColorChoice::Never)]
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
    let pattern = if args.fixed_strings {
        regex::escape(&args.pattern)
    } else {
        args.pattern.clone()
    };
    Ok(RegexBuilder::new(&pattern)
        .case_insensitive(args.ignore_case)
        .build()?)
}

/// The selected lines of `reader` as `(line number from 1, line)`.
pub fn search_lines(
    reader: impl BufRead,
    re: &Regex,
    invert: bool,
    max_count: Option<usize>,
) -> io::Result<Vec<(usize, String)>> {
    let mut found = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        if max_count.is_some_and(|max| found.len() >= max) {
            break;
        }
        let line = line?;
        if re.is_match(&line) != invert {
            found.push((i + 1, line));
        }
    }
    Ok(found)
}

/// The files to search: files as given; directories only with `recursive`,
/// walked in sorted order, skipping hidden entries (`.git`, ...).
pub fn collect_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>, GrepError> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), GrepError> {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)
            .map_err(|e| GrepError::Io(dir.to_path_buf(), e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                !p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            })
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            if !recursive {
                return Err(GrepError::IsADirectory(path.clone()));
            }
            walk(path, &mut files)?;
        } else {
            files.push(path.clone());
        }
    }
    Ok(files)
}

/// `line` with every match wrapped in ANSI bold red.
pub fn highlight(line: &str, re: &Regex) -> String {
    re.replace_all(line, "\x1b[1;31m$0\x1b[0m").into_owned()
}

/// Run `minigrep`: results on `out`, diagnostics on `err`, the exit code
/// returned (0 matched, 1 no match, 2 error -- an error wins).
pub fn run(args: &GrepArgs, out: &mut impl Write, err: &mut impl Write, is_terminal: bool) -> i32 {
    let re = match build_regex(args) {
        Ok(re) => re,
        Err(e) => {
            let _ = writeln!(err, "minigrep: {e}");
            return 2;
        }
    };
    let files = match collect_files(&args.paths, args.recursive) {
        Ok(files) => files,
        Err(e) => {
            let _ = writeln!(err, "minigrep: {e}");
            return 2;
        }
    };
    let color = match args.color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => is_terminal,
    };
    let prefix_names = files.len() > 1 || args.recursive;
    let (mut matched, mut failed) = (false, false);
    for file in &files {
        let lines = File::open(file)
            .and_then(|f| search_lines(BufReader::new(f), &re, args.invert_match, args.max_count));
        let lines = match lines {
            Ok(lines) => lines,
            Err(e) => {
                let _ = writeln!(err, "minigrep: {}: {e}", file.display());
                failed = true;
                continue;
            }
        };
        matched |= !lines.is_empty();
        let name = if prefix_names {
            format!("{}:", file.display())
        } else {
            String::new()
        };
        if args.count {
            let _ = writeln!(out, "{name}{}", lines.len());
            continue;
        }
        for (number, line) in lines {
            let number = if args.line_number {
                format!("{number}:")
            } else {
                String::new()
            };
            let text = if color && !args.invert_match {
                highlight(&line, &re)
            } else {
                line
            };
            let _ = writeln!(out, "{name}{number}{text}");
        }
    }
    if failed {
        2
    } else if matched {
        0
    } else {
        1
    }
}
