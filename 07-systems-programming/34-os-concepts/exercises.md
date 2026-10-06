# Exercises: OS Concepts

An operating system gives programs processes, files, pipes, sockets,
signals and memory mappings -- and Rust's standard library wraps most of
them (`std::process`, `std::os::unix::net`), with `nix`, `signal-hook` and
`memmap2` for the rest. This module builds the classic tools that sit
directly on those primitives: a supervisor, a shell, an IPC server, a
shared-memory cache and signal handling.

**Setup**: Unix only (macOS or Linux). The tests start real child
processes (`sh`, `sort`, `sleep`, `seq`, ...) and create sockets and files
in temporary directories -- nothing outside them, nothing needing root.

---

## Exercise 1: A Process Supervisor

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Start processes and interpret how they ended
- Implement restart policies with exponential backoff

1. `Outcome::from_status`: `Exited(code)` or `Signaled(signal)` (hint:
   `std::os::unix::process::ExitStatusExt`); `success`.
2. `backoff(attempt, base, max)`: `base * 2^attempt`, capped, without overflowing.
3. `should_restart(policy, outcome, restarts)`.
4. `Supervisor::run(sleep)`: run, record, decide, sleep (through the given
   function, so tests don't wait), repeat; honour `stop`. An unstartable
   program is an error.

**Question**: why back off at all? What happens without it when a program
crashes on startup?

---

## Exercise 2: A Small Shell

**Difficulty**: Hard
**Time**: 3 hours

**Learning Objectives**:
- Tokenize with quoting and variable expansion
- Connect processes with pipes and redirect to files
- Know which commands must be builtins, and why

1. `tokenize(line, vars)`: words, `'...'` (literal), `"..."` (with `$VAR`
   and `\"`, `\\`, `\$`), `\` escapes, `$VAR`/`$?` (unset = empty, a lone
   `$` stays), `|`, `<`, `>`, `>>`; `UnterminatedQuote`.
2. `parse(tokens)` into a `Pipeline`; `Syntax` errors for empty stages, a
   redirection without a file, `<` not on the first stage, `>` not on the last.
3. `Shell::run_line`: builtins `cd [dir]` (status 1 if missing), `pwd`,
   `export K=V`, `exit [n]` -- not allowed in a pipeline -- otherwise
   `execute`; update `$?`.
4. `execute`: spawn each stage in the shell's directory with its variables,
   stdout of one into stdin of the next, `<`/`>`/`>>`; copy the last stage's
   output to `out`; status of the last stage (127 if it can't start).
   Read the output *before* waiting, and wait for every child.

**Question**: why can't `cd` be an external program like `ls`?

---

## Exercise 3: Inter-Process Communication

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Talk to a child through pipes
- Frame messages on a stream; serve a Unix domain socket

1. `sort_via_child(lines)`: write to `sort`'s stdin, close it, read its stdout.
2. `write_message` / `read_message`: `[u32 BE length][bytes]`, `None` on a
   clean close, refuse more than `MAX_MESSAGE`.
3. `handle_command`: `SET k v` (values may contain spaces), `GET k`,
   `DEL k` -> `OK` / `VALUE v` / `NOT_FOUND` / `ERROR ...`.
4. `serve_kv`: a thread per connection, one shared store. (`KvClient` and
   `spawn_kv_server` are provided.)
5. `double_via_socketpair`: `UnixStream::pair` and a worker thread.

---

## Exercise 4: Shared Memory

**Difficulty**: Hard
**Time**: 2 hours

**Learning Objectives**:
- Map a file and share it between mappings (and processes)
- Place atomics in shared memory
- Build a lock that works across processes

The file's layout is in `ex04_shared_memory.rs` (`create`, `open` and the
raw accessors are provided).

1. `lock`: a spin lock on the header's `AtomicU32` (compare-exchange
   0 -> 1, Acquire); the guard releases it.
2. `increment` / `count` on the header's `AtomicU64`.
3. `put(key, value)` (non-zero key, at most 55 bytes, linear probing from
   `key % slots`, `Full`) and `get`, both under the lock, through raw pointers.

Then run `cargo run -p m34-os-concepts -- 4`: four *processes* increment
one counter.

**Question**: why can't the lock be a `std::sync::Mutex` in the mapped
file?

---

## Exercise 5: Signals and Resource Limits

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Handle signals safely
- Stop processes gracefully, then forcefully
- Read and set resource limits for a child

1. `flag_on(signal)` (signal-hook's `flag::register`), `wait_for(flag, timeout)`.
2. `terminate(child, grace)`: SIGTERM, wait up to `grace`, then SIGKILL.
3. `open_files_limit` (`getrlimit(RLIMIT_NOFILE)`), `run_with_file_limit`
   (`setrlimit` in `pre_exec`, so only the child is limited).

**Question**: what may a signal handler safely do, and why so little?

---

## Bonus: A Parallel Job Runner

**Difficulty**: Easy
**Time**: 30 minutes

`run_parallel(commands, max)`: at most `max` running, start the next as one
finishes, results in input order, `Exited(127)` for commands that can't start.
