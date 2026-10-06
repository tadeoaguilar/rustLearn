//! Module 28 -- 28-smart-contracts. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m28-smart-contracts -- <exercise number>
//!     cargo test -p m28-smart-contracts-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod bonus_governance;
pub mod ex01_multisig;
pub mod ex02_escrow;
pub mod ex03_staking;
pub mod ex04_security;
pub mod util;

// One program per .so: `cargo build-sbf --features escrow`, etc.
#[cfg(all(target_os = "solana", feature = "multisig"))]
mod entry {
    use crate::ex01_multisig::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "escrow"))]
mod entry {
    use crate::ex02_escrow::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "staking"))]
mod entry {
    use crate::ex03_staking::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "bank"))]
mod entry {
    use crate::ex04_security::vulnerable::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "bank-secure"))]
mod entry {
    use crate::ex04_security::secure::process;
    solana_program::entrypoint!(process);
}
