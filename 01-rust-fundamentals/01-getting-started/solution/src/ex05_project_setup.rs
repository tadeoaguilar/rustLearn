//! Exercise 5: Project Setup Best Practices.
//!
//! Most of this exercise is configuration, not code. In this crate:
//!
//! | File                 | What it does                                       |
//! |----------------------|----------------------------------------------------|
//! | `Cargo.toml [lints]` | `unsafe_code = "forbid"`, `enum_glob_use = "deny"` |
//! | `rustfmt.toml`       | formatting rules applied by `cargo fmt`            |
//! | root `.gitignore`    | keeps `target/` out of git                         |
//!
//! The function below exists to show the lints actually biting. Uncomment
//! either block and run `cargo clippy -p m01-getting-started-solution`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Fmt,
    Clippy,
    Test,
}

/// The cargo command that runs each tool.
pub fn command_for(tool: Tool) -> &'static str {
    // `use Tool::*;` here would be rejected: clippy::enum_glob_use = "deny".
    // Spelling out `Tool::Fmt` keeps it obvious where each name comes from.
    match tool {
        Tool::Fmt => "cargo fmt",
        Tool::Clippy => "cargo clippy",
        Tool::Test => "cargo test",
    }
}

// Rejected by `unsafe_code = "forbid"` -- it is a hard error, not a warning:
//
// pub fn read_raw(p: *const i32) -> i32 {
//     unsafe { *p }
// }
