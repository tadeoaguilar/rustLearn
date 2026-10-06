//! Exercise 5: signals and resource limits.
//!
//! Signals are the kernel's interrupts for processes: SIGINT (Ctrl-C),
//! SIGTERM ("please stop"), SIGKILL (stop, no choice), SIGCHLD (a child
//! exited), SIGUSR1/2 (yours to define). A handler can run between any two
//! instructions, so it may do almost nothing -- setting an atomic flag is
//! the safe pattern, and signal-hook does exactly that.
//!
//! Resource limits (`ulimit`) cap what a process may use -- open files, CPU
//! seconds, memory -- and are inherited by children.

use std::io;
use std::process::{Child, Command, Output};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use nix::sys::resource::{Resource, getrlimit, setrlimit};
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use crate::ex01_supervisor::Outcome;

/// A flag that becomes `true` when `signal` arrives.
pub fn flag_on(signal: i32) -> io::Result<Arc<AtomicBool>> {
    todo!("Exercise 5")
}

/// Wait up to `timeout` for `flag` to be set.
pub fn wait_for(flag: &AtomicBool, timeout: Duration) -> bool {
    todo!("Exercise 5")
}

/// Stop a child politely: SIGTERM, wait up to `grace` for it to exit, then
/// SIGKILL. Returns how it ended.
pub fn terminate(child: &mut Child, grace: Duration) -> io::Result<Outcome> {
    todo!("Exercise 5")
}

/// This process's (soft, hard) limit on open file descriptors.
pub fn open_files_limit() -> io::Result<(u64, u64)> {
    todo!("Exercise 5")
}

/// Run `command` with its soft open-files limit lowered to `soft` (set in
/// the child between fork and exec, so the parent is unaffected).
pub fn run_with_file_limit(command: &mut Command, soft: u64) -> io::Result<Output> {
    todo!("Exercise 5")
}
