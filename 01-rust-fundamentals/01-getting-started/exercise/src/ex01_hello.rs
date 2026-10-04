//! Exercise 1: Hello, Rust!  -- see exercises.md

use std::io::{self, BufRead, Write};

/// Return the version of the Rust compiler, e.g. "1.98.1".
///
/// Hint: this is easiest to do at *compile* time with a build script (see the
/// Bonus Challenge). Until then, returning a hard-coded string is fine.
pub fn rust_version() -> &'static str {
    todo!("Exercise 1: return the Rust version")
}

/// Return "Hello, <name>! Welcome to Rust!"
pub fn greeting(name: &str) -> String {
    todo!("Exercise 1: build the greeting")
}

/// Ask "What's your name? " on `output`, read one line from `input`,
/// then write the greeting and the Rust version line.
///
/// Why `impl BufRead` / `impl Write` instead of stdin()/stdout()? So the tests
/// can feed in fake input. `main.rs` passes the real terminal.
pub fn run(mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    todo!("Exercise 1: prompt, read a line, trim it, greet")
}
