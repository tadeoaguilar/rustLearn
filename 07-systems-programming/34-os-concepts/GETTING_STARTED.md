# Getting Started with 34 · OS Concepts

## Quick Start

All commands run from the **repository root**, on macOS or Linux.

```bash
cargo run -p m34-os-concepts-solution -- all
```

shows a supervisor restarting a crashing program with growing delays, a
shell session (quotes, pipes, redirections, `$?`, `export`, `cd`), a child
sorting through pipes, two clients sharing a Unix-socket key-value server,
four processes incrementing one counter in shared memory, a signal flag, a
`sleep` terminated with SIGTERM, a lowered file limit, and parallel jobs.

## What Is Already Here

```
34-os-concepts/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m34-os-concepts)
│   ├── ex01_supervisor.rs     #   Ex 1
│   ├── ex02_shell.rs          #   Ex 2
│   ├── ex03_ipc.rs            #   Ex 3  (KvClient, spawn_kv_server provided)
│   ├── ex04_shared_memory.rs  #   Ex 4  (create, open, raw accessors provided)
│   ├── ex05_signals.rs        #   Ex 5
│   ├── bonus_jobs.rs          #   bonus
│   └── main.rs                #   provided: demos, the worker process, an interactive shell
├── solution/                  # ← REFERENCE (m34-os-concepts-solution) + ANSWERS.md
└── tests/                     # ← 22 tests (m34-os-concepts-tests)
```

## The Commands You Need

```bash
cargo test -p m34-os-concepts-tests --features mine ex2_
cargo test -p m34-os-concepts-tests --features mine
cargo test -p m34-os-concepts-tests                    # the solution: always green
cargo run  -p m34-os-concepts -- shell                 # your shell, interactively
```

Inside your shell, try `ls | wc -l`, `echo $HOME > home.txt`, `cat < home.txt`,
`sh -c 'exit 3'; echo $?`, `cd ..`, `exit`.

## If You Get Stuck

1. **A pipeline hangs** -- you waited for a child before reading its output, or kept a copy of a pipe's write end open (the reader then never sees EOF).
2. **`sort_via_child` hangs** -- drop the child's stdin before reading: that's the EOF `sort` waits for.
3. **`$?` is always 0** -- update `vars["?"]` after every command, builtins included.
4. **"Address already in use" binding the socket** -- a file exists at that path; the tests use fresh temporary directories.
5. **Shared counter lower than expected** -- use `fetch_add` on the mapped `AtomicU64`, not load + store.
6. **`terminate` never returns** -- poll `try_wait` in a loop; `wait` blocks forever on a process ignoring SIGTERM.
7. Compare with `solution/src/` -- same file and function names.
