//! Module 34 -- OS Concepts. Reference solution. Unix only (macOS, Linux).
//!
//! | File                     | Exercise |
//! |--------------------------|----------|
//! | `ex01_supervisor.rs`     | 1  a process supervisor: restart policies, backoff |
//! | `ex02_shell.rs`          | 2  a small shell: tokenizer, pipes, redirection, builtins |
//! | `ex03_ipc.rs`            | 3  IPC: pipes, a Unix-socket key-value server, socketpair |
//! | `ex04_shared_memory.rs`  | 4  shared memory: a cache in a memory-mapped file |
//! | `ex05_signals.rs`        | 5  signals and resource limits |
//! | `bonus_jobs.rs`          | bonus: a parallel job runner |

pub mod bonus_jobs;
pub mod ex01_supervisor;
pub mod ex02_shell;
pub mod ex03_ipc;
pub mod ex04_shared_memory;
pub mod ex05_signals;
