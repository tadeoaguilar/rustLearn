# 34 · OS Concepts

## Overview

Below every runtime are the operating system's primitives: processes that
start, exit and get signalled; file descriptors for files, pipes and
sockets; memory that can be mapped and shared. Systems programs -- init
systems, shells, databases, container runtimes -- are written directly
against them. This module builds small versions of five such programs on
Unix, using `std` where it reaches and `nix`, `signal-hook` and `memmap2`
where it doesn't.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Supervisor | Exit statuses, signals, restart policies, backoff |
| 2 | Shell | Tokenizing, pipes, redirection, builtins |
| 3 | IPC | Pipes, Unix domain sockets, socketpair, framing |
| 4 | Shared memory | `mmap`, atomics in shared pages, a cross-process lock |
| 5 | Signals, limits | Safe handlers, graceful termination, `rlimit` |
| Bonus | Job runner | Bounded parallelism over processes |

## Key Concepts

### Processes

```rust
let mut child = Command::new("sort").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
// ... write to child.stdin, drop it (EOF), read child.stdout ...
let status = child.wait()?;            // always wait: an unwaited child is a zombie
status.code()                          // Some(n) if it exited
status.signal()                        // Some(sig) if a signal killed it (ExitStatusExt)
```

### File descriptors and pipes

A pipe is two descriptors: write bytes into one, read them from the other.
`a | b` is a pipe whose write end is `a`'s stdout and read end `b`'s stdin;
`Stdio::from(child.stdout.take())` passes it on. A reader sees EOF only when
*every* write end is closed.

### Unix domain sockets

```rust
let listener = UnixListener::bind("/tmp/app.sock")?;    // a path, local only
let stream = UnixStream::connect("/tmp/app.sock")?;
let (a, b) = UnixStream::pair()?;                       // no path: socketpair(2)
```

Like TCP but local, faster, and permission-controlled by the filesystem.

### Shared memory

Two `MAP_SHARED` mappings of one file are the same physical pages. Atomic
operations work across processes on such memory; ordinary Rust references
into it don't hold their promises (another process can change the bytes),
so access it through raw pointers under a lock.

### Signals

| Signal | Default | Typical use |
|---|---|---|
| SIGINT | terminate | Ctrl-C |
| SIGTERM | terminate | "please shut down" (catchable) |
| SIGKILL | terminate | cannot be caught or ignored |
| SIGCHLD | ignore | a child exited |
| SIGUSR1/2 | terminate | application-defined |

## Common Pitfalls

1. **Not waiting for children** -- zombies accumulate until the parent exits
2. **Waiting before reading a child's output** -- the pipe fills and both block forever
3. **Forgetting to close the write end of a pipe** -- the reader never sees EOF
4. **Doing real work in a signal handler** -- set a flag instead
5. **Assuming one `read` returns one message** -- frame it
6. **References into shared memory** -- another process can change the bytes under you
7. **A stale socket file** -- `bind` fails if the path exists; remove it first

## Running This Module

Unix only (macOS, Linux).

```bash
cargo run  -p m34-os-concepts -- 1                     # your code (1-5, bonus, all)
cargo test -p m34-os-concepts-tests --features mine    # test your code
cargo run  -p m34-os-concepts-solution -- all          # a supervisor, a shell session, IPC, 4 processes sharing memory, signals
cargo run  -p m34-os-concepts-solution -- shell        # an interactive shell
cargo test -p m34-os-concepts-tests                    # 22 tests against the solution
```

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a process supervisor, a shared memory
cache, an IPC message queue, a simple shell).

- **Unix only**: Windows has different primitives (named pipes, job
  objects); the crates in this module compile only on Unix.
- **No `fork`**: the tests use `std::process::Command` (fork + exec, done
  safely by `std`); the shared-memory demo starts real separate processes
  by re-running its own binary. Calling `fork` directly in a multi-threaded
  program (like a test harness) is a trap -- only async-signal-safe calls
  are allowed in the child.
- **Not covered**: process scheduling and virtual memory internals (no code
  to write), namespaces and cgroups (Linux-only, need privileges).

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[35 · Memory Management](../35-memory-management/)
