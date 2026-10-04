//! Exercise 1: Hello, Rust!
//!
//! The interesting decision is the function signature. `run` accepts any
//! reader and any writer, so `main` can hand it the real terminal while the
//! tests hand it a byte slice and a `Vec<u8>`.

use std::io::{self, BufRead, Write};

/// The compiler version captured by `build.rs` at compile time.
pub fn rust_version() -> &'static str {
    env!("RUSTC_VERSION")
}

/// Pure function: no I/O, trivially testable.
pub fn greeting(name: &str) -> String {
    format!("Hello, {name}! Welcome to Rust!")
}

/// Asks for a name on `input`, writes the greeting to `output`.
pub fn run(mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    write!(output, "What's your name? ")?;
    // stdout is line-buffered: without flush the prompt may not appear
    // before read_line blocks.
    output.flush()?;

    let mut name = String::new();
    input.read_line(&mut name)?;
    // read_line keeps the trailing '\n' -- trim it or the greeting breaks
    // across two lines.
    let name = name.trim();
    let name = if name.is_empty() { "stranger" } else { name };

    writeln!(output, "{}", greeting(name))?;
    writeln!(output, "You're using Rust version: {}", rust_version())?;
    Ok(())
}
