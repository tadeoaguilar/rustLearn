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
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut chars = line.chars().peekable();
    let expand = |chars: &mut std::iter::Peekable<std::str::Chars>, word: &mut String| {
        let mut name = String::new();
        if chars.peek() == Some(&'?') {
            chars.next();
            name.push('?');
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
        }
        if name.is_empty() {
            word.push('$');
        } else {
            word.push_str(vars.get(&name).map_or("", String::as_str));
        }
    };
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' => {
                if in_word {
                    tokens.push(Token::Word(std::mem::take(&mut word)));
                    in_word = false;
                }
            }
            '|' | '<' | '>' => {
                if in_word {
                    tokens.push(Token::Word(std::mem::take(&mut word)));
                    in_word = false;
                }
                tokens.push(match c {
                    '|' => Token::Pipe,
                    '<' => Token::In,
                    _ if chars.peek() == Some(&'>') => {
                        chars.next();
                        Token::Append
                    }
                    _ => Token::Out,
                });
            }
            '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(c) => word.push(c),
                        None => return Err(ShellError::UnterminatedQuote),
                    }
                }
            }
            '"' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some(c @ ('"' | '\\' | '$')) => word.push(c),
                            Some(c) => {
                                word.push('\\');
                                word.push(c);
                            }
                            None => return Err(ShellError::UnterminatedQuote),
                        },
                        Some('$') => expand(&mut chars, &mut word),
                        Some(c) => word.push(c),
                        None => return Err(ShellError::UnterminatedQuote),
                    }
                }
            }
            '\\' => {
                in_word = true;
                if let Some(c) = chars.next() {
                    word.push(c);
                }
            }
            '$' => {
                in_word = true;
                expand(&mut chars, &mut word);
            }
            c => {
                in_word = true;
                word.push(c);
            }
        }
    }
    if in_word {
        tokens.push(Token::Word(word));
    }
    Ok(tokens)
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
    let mut pipeline = Pipeline::default();
    let mut current: Vec<String> = Vec::new();
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        match token {
            Token::Word(w) => current.push(w.clone()),
            Token::Pipe => {
                if current.is_empty() || pipeline.output.is_some() {
                    return Err(ShellError::Syntax("|".into()));
                }
                pipeline.stages.push(std::mem::take(&mut current));
            }
            Token::In | Token::Out | Token::Append => {
                let symbol = match token {
                    Token::In => "<",
                    Token::Out => ">",
                    _ => ">>",
                };
                let Some(Token::Word(file)) = iter.next() else {
                    return Err(ShellError::Syntax(symbol.into()));
                };
                if *token == Token::In {
                    if !pipeline.stages.is_empty() {
                        return Err(ShellError::Syntax("<".into()));
                    }
                    pipeline.input = Some(file.into());
                } else {
                    pipeline.output = Some((file.into(), *token == Token::Append));
                }
            }
        }
    }
    if current.is_empty() {
        if pipeline.stages.is_empty() && pipeline.input.is_none() && pipeline.output.is_none() {
            return Ok(pipeline); // an empty line
        }
        return Err(ShellError::Syntax("end of line".into()));
    }
    pipeline.stages.push(current);
    Ok(pipeline)
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
        let mut vars: HashMap<String, String> = std::env::vars().collect();
        vars.insert("?".into(), "0".into());
        Shell {
            cwd,
            vars,
            last_status: 0,
        }
    }

    /// Run one line; the last stage's stdout (when not redirected) is
    /// copied to `out`.
    pub fn run_line(&mut self, line: &str, out: &mut impl Write) -> Result<Control, ShellError> {
        let tokens = tokenize(line, &self.vars)?;
        let pipeline = parse(&tokens)?;
        if pipeline.stages.is_empty() {
            return Ok(Control::Continue(self.last_status));
        }
        let control = match self.builtin(&pipeline, out)? {
            Some(control) => control,
            None => Control::Continue(
                self.execute(&pipeline, out)
                    .map_err(|e| ShellError::Exec(e.to_string()))?,
            ),
        };
        if let Control::Continue(status) = control {
            self.last_status = status;
            self.vars.insert("?".into(), status.to_string());
        }
        Ok(control)
    }

    /// `cd`, `pwd`, `export`, `exit` -- only as a single stage.
    fn builtin(
        &mut self,
        pipeline: &Pipeline,
        out: &mut impl Write,
    ) -> Result<Option<Control>, ShellError> {
        let argv = &pipeline.stages[0];
        let name = argv[0].as_str();
        if !matches!(name, "cd" | "pwd" | "export" | "exit") {
            return Ok(None);
        }
        if pipeline.stages.len() > 1 {
            return Err(ShellError::Syntax(format!("{name} in a pipeline")));
        }
        let control = match name {
            "cd" => {
                let target = match argv.get(1) {
                    Some(dir) => self.cwd.join(dir),
                    None => {
                        PathBuf::from(self.vars.get("HOME").cloned().unwrap_or_else(|| "/".into()))
                    }
                };
                match target.canonicalize() {
                    Ok(dir) if dir.is_dir() => {
                        self.cwd = dir;
                        Control::Continue(0)
                    }
                    _ => {
                        let _ = writeln!(out, "cd: no such directory: {}", target.display());
                        Control::Continue(1)
                    }
                }
            }
            "pwd" => {
                let _ = writeln!(out, "{}", self.cwd.display());
                Control::Continue(0)
            }
            "export" => {
                for assignment in &argv[1..] {
                    let (k, v) = assignment
                        .split_once('=')
                        .ok_or_else(|| ShellError::Syntax(assignment.clone()))?;
                    self.vars.insert(k.into(), v.into());
                }
                Control::Continue(0)
            }
            _ => Control::Exit(
                argv.get(1)
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(self.last_status),
            ),
        };
        Ok(Some(control))
    }

    /// Spawn every stage, wiring stdout to the next stdin; wait for all;
    /// the status is the last stage's (127 if it couldn't be started).
    pub fn execute(&mut self, pipeline: &Pipeline, out: &mut impl Write) -> io::Result<i32> {
        let mut children: Vec<Child> = Vec::new();
        let mut previous_stdout: Option<Stdio> = match &pipeline.input {
            Some(path) => Some(Stdio::from(File::open(self.cwd.join(path))?)),
            None => None,
        };
        let last = pipeline.stages.len() - 1;
        for (i, argv) in pipeline.stages.iter().enumerate() {
            let mut command = Command::new(&argv[0]);
            command
                .args(&argv[1..])
                .current_dir(&self.cwd)
                .env_clear()
                .envs(self.vars.iter().filter(|(k, _)| *k != "?"));
            if let Some(stdin) = previous_stdout.take() {
                command.stdin(stdin);
            }
            if i == last {
                match &pipeline.output {
                    Some((path, append)) => {
                        let file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(*append)
                            .truncate(!*append)
                            .open(self.cwd.join(path))?;
                        command.stdout(file);
                    }
                    None => {
                        command.stdout(Stdio::piped());
                    }
                }
            } else {
                command.stdout(Stdio::piped());
            }
            match command.spawn() {
                Ok(mut child) => {
                    if i != last {
                        previous_stdout = child.stdout.take().map(Stdio::from);
                    }
                    children.push(child);
                }
                Err(e) => {
                    for mut child in children {
                        let _ = child.wait();
                    }
                    writeln!(out, "{}: {e}", argv[0])?;
                    return Ok(127);
                }
            }
        }
        // Read the last stage's output before waiting, so a full pipe can't
        // deadlock -- and wait for every child even if reading fails, so none
        // is left a zombie.
        let mut last_child = children.pop().expect("at least one stage");
        let mut buf = Vec::new();
        let read = match last_child.stdout.take() {
            Some(mut stdout) => stdout.read_to_end(&mut buf).map(|_| ()),
            None => Ok(()),
        };
        let mut first_error = None;
        for mut child in children {
            if let Err(e) = child.wait() {
                first_error.get_or_insert(e);
            }
        }
        let status = last_child.wait()?;
        if let Some(e) = first_error {
            return Err(e);
        }
        read?;
        out.write_all(&buf)?;
        Ok(status
            .code()
            .unwrap_or(128 + std::os::unix::process::ExitStatusExt::signal(&status).unwrap_or(0)))
    }
}
