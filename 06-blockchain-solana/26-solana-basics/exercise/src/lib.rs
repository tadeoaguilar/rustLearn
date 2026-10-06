//! Module 26 -- 26-solana-basics. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m26-solana-basics -- <exercise number>
//!     cargo test -p m26-solana-basics-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

#[cfg(not(target_os = "solana"))]
pub mod bonus_client;
#[cfg(not(target_os = "solana"))]
pub mod ex01_wallet;
#[cfg(not(target_os = "solana"))]
pub mod ex02_transfers;
pub mod ex03_hello;
pub mod ex04_notes_state;
pub mod ex05_notes;
pub mod ex06_vault;

// One program per .so: `cargo build-sbf --features notes` builds the notes
// program. See the phase's GETTING_STARTED.md.
#[cfg(all(target_os = "solana", feature = "hello"))]
mod entry {
    use crate::ex03_hello::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "notes"))]
mod entry {
    use crate::ex05_notes::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "vault"))]
mod entry {
    use crate::ex06_vault::process;
    solana_program::entrypoint!(process);
}
