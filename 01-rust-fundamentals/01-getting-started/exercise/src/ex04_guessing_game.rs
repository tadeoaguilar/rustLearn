//! Exercise 4: Number Guessing Game.  -- see exercises.md

use std::cmp::Ordering;
use std::io::{self, BufRead, Write};

pub const MIN: u32 = 1;
pub const MAX: u32 = 100;

/// A random number between MIN and MAX inclusive.
/// Hint: `use rand::Rng;` then `rand::thread_rng().gen_range(MIN..=MAX)`.
pub fn secret_number() -> u32 {
    todo!("Exercise 4: generate the secret")
}

/// Compare the guess with the secret. Hint: integers have a `.cmp()` method.
pub fn check_guess(guess: u32, secret: u32) -> Ordering {
    todo!("Exercise 4")
}

/// Play one game. Print "Too small!", "Too big!" or "You got it in N guesses!".
/// Ignore input that is not a number between MIN and MAX (don't count it).
/// Return Some(number of guesses), or None if `input` runs out first
/// (`read_line` returns Ok(0) at end of input).
pub fn play(
    secret: u32,
    mut input: impl BufRead,
    mut output: impl Write,
) -> io::Result<Option<u32>> {
    todo!("Exercise 4: the game loop")
}
