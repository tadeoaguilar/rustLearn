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
    let max = max.max(1);
    let mut results: Vec<Option<Outcome>> = vec![None; commands.len()];
    let mut running: Vec<(usize, Child)> = Vec::new();
    let mut next = 0;
    while next < commands.len() || !running.is_empty() {
        while running.len() < max && next < commands.len() {
            let argv = &commands[next];
            match Command::new(&argv[0])
                .args(&argv[1..])
                .stdout(Stdio::null())
                .spawn()
            {
                Ok(child) => running.push((next, child)),
                Err(_) => results[next] = Some(Outcome::Exited(127)),
            }
            next += 1;
        }
        let mut i = 0;
        while i < running.len() {
            if let Some(status) = running[i].1.try_wait()? {
                let (index, _) = running.swap_remove(i);
                results[index] = Some(Outcome::from_status(status));
            } else {
                i += 1;
            }
        }
        if !running.is_empty() {
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    Ok(results
        .into_iter()
        .map(|r| r.expect("every job finished"))
        .collect())
}
