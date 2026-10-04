//! Exercise 4: Number Guessing Game.
//!
//! The game is split into three layers, which is the habit worth learning
//! here more than the game itself:
//!
//! 1. `check_guess`     -- pure logic, no I/O, no randomness
//! 2. `play`            -- the loop, talking to any reader/writer
//! 3. `secret_number`   -- the only part that touches `rand`
//!
//! Only (3) is untestable, and it is one line long.

use rand::Rng;
use std::cmp::Ordering;
use std::io::{self, BufRead, Write};

pub const MIN: u32 = 1;
pub const MAX: u32 = 100;

/// A random number in `MIN..=MAX`.
pub fn secret_number() -> u32 {
    rand::thread_rng().gen_range(MIN..=MAX)
}

/// Compares a guess to the secret. `Ordering` is exactly the three-way answer
/// we need: Less ("too small"), Greater ("too big"), Equal ("you got it").
pub fn check_guess(guess: u32, secret: u32) -> Ordering {
    guess.cmp(&secret)
}

/// Plays one game against `secret`. Returns the number of valid guesses it
/// took, or `None` if the input ran out before the number was found.
pub fn play(
    secret: u32,
    mut input: impl BufRead,
    mut output: impl Write,
) -> io::Result<Option<u32>> {
    writeln!(output, "Guess the number!")?;
    let mut attempts = 0;

    loop {
        write!(output, "Please input your guess: ")?;
        output.flush()?;

        let mut line = String::new();
        // read_line returns Ok(0) at end of input (Ctrl-D, or end of a pipe).
        if input.read_line(&mut line)? == 0 {
            writeln!(output)?;
            return Ok(None);
        }

        // `.trim()` removes the '\n'; without it every parse fails.
        let guess: u32 = match line.trim().parse() {
            Ok(n) if (MIN..=MAX).contains(&n) => n,
            Ok(_) => {
                writeln!(output, "Please pick a number between {MIN} and {MAX}.")?;
                continue;
            }
            Err(_) => {
                writeln!(output, "That's not a number, try again.")?;
                continue;
            }
        };
        attempts += 1;

        match check_guess(guess, secret) {
            Ordering::Less => writeln!(output, "Too small!")?,
            Ordering::Greater => writeln!(output, "Too big!")?,
            Ordering::Equal => {
                let plural = if attempts == 1 { "guess" } else { "guesses" };
                writeln!(output, "You got it in {attempts} {plural}!")?;
                return Ok(Some(attempts));
            }
        }
    }
}
