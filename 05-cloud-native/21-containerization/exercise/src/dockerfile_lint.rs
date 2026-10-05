//! Exercise 6: a Dockerfile linter.
//!
//! Checking the rules yourself is the best way to learn them. Each rule is a
//! mistake that's easy to make and costs size, build time or security.
//! (`hadolint` is the real-world tool.)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// 1-based line where the instruction starts.
    pub line: usize,
    /// Upper-case: FROM, RUN, COPY...
    pub keyword: String,
    /// Everything after the keyword, continuation lines joined with spaces.
    pub args: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    /// `FROM rust` or `FROM rust:latest`: the build changes under you.
    UnpinnedBaseImage,
    /// One stage: the compiler and sources end up in the image you ship.
    SingleStage,
    /// No `USER` (or `USER root`) in the final stage.
    RunsAsRoot,
    /// No `HEALTHCHECK` in the final stage.
    NoHealthcheck,
    /// `cargo build` with neither cargo-chef nor a cache mount: every source
    /// change recompiles every dependency.
    NoDependencyCaching,
    /// `ADD` of local files: `COPY` says what it does (ADD also unpacks tars
    /// and downloads URLs).
    AddForLocalFiles,
    /// `CMD app` / `ENTRYPOINT app` (shell form): `/bin/sh` becomes PID 1
    /// and SIGTERM never reaches the app; and `scratch` has no shell.
    ShellFormCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub line: usize,
    pub rule: Rule,
    pub message: String,
}

/// Splits a Dockerfile into instructions: skips blank lines and comments,
/// joins lines ending in `\`.
pub fn parse(text: &str) -> Vec<Instruction> {
    todo!("Exercise 6")
}

fn split(line: usize, text: &str) -> Instruction {
    todo!("Exercise 6")
}

/// The image of a FROM line, skipping flags like `--platform=...`.
fn from_image(args: &str) -> &str {
    todo!("Exercise 6")
}

/// The stage name: `FROM image AS name`.
fn from_alias(args: &str) -> Option<String> {
    todo!("Exercise 6")
}

fn is_unpinned(image: &str) -> bool {
    todo!("Exercise 6")
}

pub fn lint(text: &str) -> Vec<Finding> {
    todo!("Exercise 6")
}
