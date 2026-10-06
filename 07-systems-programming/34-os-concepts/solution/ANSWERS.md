# Answers · 34 OS Concepts

## Exercise 1: Why back off?

A program that crashes at startup (a missing config file, a port already in
use) would otherwise be restarted thousands of times a second: one CPU core
spent forking, the logs flooded, and whatever it depends on (a database it
connects to first) hammered. Doubling the wait spreads retries out quickly
while still recovering fast from a one-off failure; the cap keeps the wait
bounded, and the restart limit eventually reports the failure to a human.
Kubernetes calls the state `CrashLoopBackOff`.

## Exercise 2: Why must `cd` be a builtin?

The current directory is per-process state, and a child can't change its
parent's state. An external `cd` would change *its own* directory and exit;
the shell would be exactly where it was. The same goes for `export` (the
environment), `exit` (the shell's own life) and `ulimit` -- anything that
must change the shell itself.

## Exercise 4: Why not `std::sync::Mutex`?

Two reasons. A `std` mutex isn't a plain value you can place at an address
two processes share: on some platforms it's a pointer to a heap allocation,
which means nothing in another process's address space. And it's only
specified for threads of one process. A lock in shared memory must be plain
data that works through any mapping -- an atomic integer with
compare-and-swap (or a `pthread_mutex_t` initialised with
`PTHREAD_PROCESS_SHARED`). The trade-off of our spin lock: if a process
dies while holding it, nobody releases it (robust locks or lease timeouts
fix that).

## Exercise 5: What may a signal handler do?

Almost nothing: it can interrupt the program between any two instructions
-- in the middle of `malloc`, holding a lock, halfway through writing a
buffer. Calling anything that might take that lock or touch that state
deadlocks or corrupts it. POSIX lists the *async-signal-safe* functions
(`write`, `_exit`, ...); `printf` and `malloc` aren't among them. The safe
pattern is to set an atomic flag (or write a byte to a pipe) and let normal
code notice it -- which is what `signal_hook::flag::register` does.
