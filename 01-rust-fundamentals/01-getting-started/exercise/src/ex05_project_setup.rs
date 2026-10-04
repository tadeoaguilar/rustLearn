//! Exercise 5: Project Setup Best Practices.  -- see exercises.md
//!
//! Most of this exercise is configuration: add [lints] to ../Cargo.toml and
//! create ../rustfmt.toml. Then complete the function below WITHOUT using
//! `use Tool::*;` (which your new clippy lint should reject -- try it!).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Fmt,
    Clippy,
    Test,
}

/// "cargo fmt", "cargo clippy" or "cargo test".
pub fn command_for(tool: Tool) -> &'static str {
    todo!("Exercise 5")
}
