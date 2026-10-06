//! Bonus: a parallel job runner (like `xargs -P` or `make -j`).
//!
//! Start up to `max` commands at once; as each finishes, start the next.
//! Results come back in input order, whatever order the jobs finished in.

use std::io;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::ex01_supervisor::Outcome;

/// Run every command (`argv` vectors) with at most `max` running at once.
/// A command that can't be started counts as `Exited(127)`.
pub fn run_parallel(commands: &[Vec<String>], max: usize) -> io::Result<Vec<Outcome>> {
    todo!("Bonus")
}
