//! Exercise 9: Temperature Converter.

use std::io::{self, BufRead, Write};

pub fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

/// Converts `value` given in `unit` ('C' or 'F', any case) to the other unit.
/// Returns the converted value and its unit, or None for an unknown unit.
pub fn convert(value: f64, unit: char) -> Option<(f64, char)> {
    match unit.to_ascii_uppercase() {
        'C' => Some((celsius_to_fahrenheit(value), 'F')),
        'F' => Some((fahrenheit_to_celsius(value), 'C')),
        _ => None,
    }
}

/// Prompts for a value and a unit, prints the conversion.
pub fn run(mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    let value: f64 = loop {
        write!(output, "Temperature value: ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(());
        }
        match line.trim().parse() {
            Ok(v) => break v,
            Err(_) => writeln!(output, "Not a number, try again.")?,
        }
    };

    write!(output, "Unit (C or F): ")?;
    output.flush()?;
    let mut line = String::new();
    input.read_line(&mut line)?;
    let unit = line.trim().chars().next().unwrap_or(' ');

    match convert(value, unit) {
        Some((converted, to)) => writeln!(
            output,
            "{value}°{} = {converted:.1}°{to}",
            unit.to_ascii_uppercase()
        )?,
        None => writeln!(output, "Unknown unit '{unit}', expected C or F")?,
    }
    Ok(())
}
