//! Exercise 9: Temperature Converter.

use std::io::{self, BufRead, Write};

pub fn fahrenheit_to_celsius(f: f64) -> f64 {
    todo!("Exercise 9")
}

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    todo!("Exercise 9")
}

/// Converts `value` given in `unit` ('C' or 'F', any case) to the other unit.
/// Returns the converted value and its unit, or None for an unknown unit.
pub fn convert(value: f64, unit: char) -> Option<(f64, char)> {
    todo!("Exercise 9")
}

/// Prompts for a value and a unit, prints the conversion.
pub fn run(mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    todo!("Exercise 9")
}
