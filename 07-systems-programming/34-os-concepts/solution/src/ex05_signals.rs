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
    let flag = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal, flag.clone())?;
    Ok(flag)
}

/// Wait up to `timeout` for `flag` to be set.
pub fn wait_for(flag: &AtomicBool, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if flag.load(Ordering::SeqCst) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    flag.load(Ordering::SeqCst)
}

/// Stop a child politely: SIGTERM, wait up to `grace` for it to exit, then
/// SIGKILL. Returns how it ended.
pub fn terminate(child: &mut Child, grace: Duration) -> io::Result<Outcome> {
    let pid = Pid::from_raw(child.id() as i32);
    kill(pid, Signal::SIGTERM).map_err(io::Error::from)?;
    let start = Instant::now();
    while start.elapsed() < grace {
        if let Some(status) = child.try_wait()? {
            return Ok(Outcome::from_status(status));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    child.kill()?; // SIGKILL
    Ok(Outcome::from_status(child.wait()?))
}

/// This process's (soft, hard) limit on open file descriptors.
pub fn open_files_limit() -> io::Result<(u64, u64)> {
    let (soft, hard) = getrlimit(Resource::RLIMIT_NOFILE).map_err(io::Error::from)?;
    // `rlim_t` is u64 on Linux and macOS, but not on every Unix.
    #[allow(clippy::unnecessary_cast)]
    Ok((soft as u64, hard as u64))
}

/// Run `command` with its soft open-files limit lowered to `soft` (set in
/// the child between fork and exec, so the parent is unaffected).
pub fn run_with_file_limit(command: &mut Command, soft: u64) -> io::Result<Output> {
    let (_, hard) = open_files_limit()?;
    let soft = soft.min(hard);
    // SAFETY: the closure runs in the child after fork, before exec; it only
    // calls setrlimit (async-signal-safe) and allocates nothing.
    unsafe {
        std::os::unix::process::CommandExt::pre_exec(command, move || {
            setrlimit(Resource::RLIMIT_NOFILE, soft as _, hard as _).map_err(io::Error::from)
        });
    }
    command.output()
}
