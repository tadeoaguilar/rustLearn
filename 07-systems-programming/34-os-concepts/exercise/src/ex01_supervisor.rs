//! Exercise 1: a process supervisor.
//!
//! systemd, runit, Kubernetes' kubelet and every `pm2`-style tool do the
//! same job: start a program, notice when it exits, decide whether to start
//! it again, and avoid a crash loop that burns the CPU by waiting longer
//! after each failure (exponential backoff), giving up after too many.

use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// How a process ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// It called `exit(code)`.
    Exited(i32),
    /// A signal killed it (`kill -9` is `Signaled(9)`).
    Signaled(i32),
}

impl Outcome {
    pub fn from_status(status: ExitStatus) -> Outcome {
        todo!("Exercise 1")
    }

    pub fn success(&self) -> bool {
        todo!("Exercise 1")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restart {
    Never,
    /// Only after a non-zero exit or a signal.
    OnFailure,
    Always,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub restart: Restart,
    /// Give up after this many restarts.
    pub max_restarts: u32,
    pub backoff_base: Duration,
    pub backoff_max: Duration,
}

/// The wait before restart number `attempt` (0-based): `base * 2^attempt`,
/// capped at `max` (and without overflowing for large attempts).
pub fn backoff(attempt: u32, base: Duration, max: Duration) -> Duration {
    todo!("Exercise 1")
}

/// Whether to restart after `outcome`, having restarted `restarts` times.
pub fn should_restart(policy: &Policy, outcome: Outcome, restarts: u32) -> bool {
    todo!("Exercise 1")
}

/// One run of the child, and how long the supervisor waited after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    pub outcome: Outcome,
    pub delay_before_restart: Option<Duration>,
}

pub struct Supervisor {
    pub program: String,
    pub args: Vec<String>,
    pub policy: Policy,
    /// Set (e.g. from a SIGTERM handler) to stop restarting.
    pub stop: Arc<AtomicBool>,
}

impl Supervisor {
    pub fn new(program: &str, args: &[&str], policy: Policy) -> Self {
        todo!("Exercise 1")
    }

    /// Run the child until the policy says stop (or `stop` is set), calling
    /// `sleep` for each backoff (tests pass a recorder instead of sleeping).
    /// Returns the record of every run. Fails if the program can't start.
    pub fn run(&self, sleep: &mut dyn FnMut(Duration)) -> io::Result<Vec<RunRecord>> {
        todo!("Exercise 1")
    }
}
