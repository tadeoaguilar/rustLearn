//! Exercise 2: a small shell.
//!
//! A shell is a parser and a process launcher. `ls -l | grep rs > out.txt`
//! becomes two child processes whose stdout and stdin are joined by a pipe,
//! the last one's output going to a file. Some commands can't be programs:
//! `cd` must change the *shell's* directory, `exit` must end the shell --
//! those are builtins.
//!
//! Supported: words, `'single'` and `"double"` quotes, `\` escapes, `$VAR`
//! and `$?` (outside single quotes), `|`, `<`, `>`, `>>`, and the builtins
//! `cd`, `pwd`, `export NAME=value` and `exit [code]`.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Pipe,
    /// `<`
    In,
    /// `>`
    Out,
    /// `>>`
    Append,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ShellError {
    #[error("unterminated quote")]
    UnterminatedQuote,
    #[error("syntax error near {0:?}")]
    Syntax(String),
    #[error("{0}")]
    Exec(String),
}

/// Split a line into tokens, expanding variables from `vars` (`$?` comes
/// from `vars["?"]`).
pub fn tokenize(line: &str, vars: &HashMap<String, String>) -> Result<Vec<Token>, ShellError> {
    todo!("Exercise 2")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pipeline {
    /// Each stage's argv.
    pub stages: Vec<Vec<String>>,
    /// `< file` (for the first stage).
    pub input: Option<PathBuf>,
    /// `> file` / `>> file` (for the last stage), and whether to append.
    pub output: Option<(PathBuf, bool)>,
}

/// Group tokens into a pipeline. Errors: an empty stage (`| x`, `x |`,
/// `x || y`), a redirection without a file, `<` after the first stage or
/// `>` before the last.
pub fn parse(tokens: &[Token]) -> Result<Pipeline, ShellError> {
    todo!("Exercise 2")
}

/// What the caller should do after a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Continue(i32),
    Exit(i32),
}

#[derive(Debug, Clone)]
pub struct Shell {
    pub cwd: PathBuf,
    pub vars: HashMap<String, String>,
    pub last_status: i32,
}

impl Shell {
    pub fn new(cwd: PathBuf) -> Self {
        todo!("Exercise 2")
    }

    /// Run one line; the last stage's stdout (when not redirected) is
    /// copied to `out`.
    pub fn run_line(&mut self, line: &str, out: &mut impl Write) -> Result<Control, ShellError> {
        todo!("Exercise 2")
    }

    /// `cd`, `pwd`, `export`, `exit` -- only as a single stage.
    fn builtin(
        &mut self,
        pipeline: &Pipeline,
        out: &mut impl Write,
    ) -> Result<Option<Control>, ShellError> {
        todo!("Exercise 2")
    }

    /// Spawn every stage, wiring stdout to the next stdin; wait for all;
    /// the status is the last stage's (127 if it couldn't be started).
    pub fn execute(&mut self, pipeline: &Pipeline, out: &mut impl Write) -> io::Result<i32> {
        todo!("Exercise 2")
    }
}
