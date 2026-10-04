//! Exercise 7: Calculator Program.

use std::io::{self, BufRead, Write};

/// Parses "5 + 3" into (5.0, '+', 3.0).
///
/// `split_whitespace` handles any amount of spacing. Every step that can fail
/// returns `None` through `?` -- yes, `?` works on `Option` as well as `Result`.
pub fn parse_operation(input: &str) -> Option<(f64, char, f64)> {
    let mut parts = input.split_whitespace();
    let a: f64 = parts.next()?.parse().ok()?;
    let op_text = parts.next()?;
    let b: f64 = parts.next()?.parse().ok()?;

    if parts.next().is_some() {
        return None; // "1 + 2 + 3" is not supported
    }

    let mut op_chars = op_text.chars();
    let op = op_chars.next()?;
    if op_chars.next().is_some() || !matches!(op, '+' | '-' | '*' | '/') {
        return None;
    }
    Some((a, op, b))
}

/// `None` for division by zero (and for an unknown operator).
pub fn calculate(a: f64, op: char, b: f64) -> Option<f64> {
    match op {
        '+' => Some(a + b),
        '-' => Some(a - b),
        '*' => Some(a * b),
        '/' if b == 0.0 => None,
        '/' => Some(a / b),
        _ => None,
    }
}

/// The interactive loop. Reads lines until "quit" or end of input.
pub fn run(input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    writeln!(output, "Calculator (type 'quit' to exit)")?;
    write!(output, "> ")?;
    output.flush()?;

    for line in input.lines() {
        let line = line?;
        let line = line.trim();
        if line.eq_ignore_ascii_case("quit") {
            break;
        }

        match parse_operation(line) {
            None => writeln!(output, "Error: expected something like '5 + 3'")?,
            Some((a, op, b)) => match calculate(a, op, b) {
                // f64's Display prints 8.0 as "8", matching the expected output.
                Some(result) => writeln!(output, "Result: {result}")?,
                None => writeln!(output, "Error: Division by zero")?,
            },
        }
        write!(output, "> ")?;
        output.flush()?;
    }

    writeln!(output, "Goodbye!")?;
    Ok(())
}
