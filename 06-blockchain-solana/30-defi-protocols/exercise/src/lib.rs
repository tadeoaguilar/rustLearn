//! Module 30 -- 30-defi-protocols. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m30-defi-protocols -- <exercise number>
//!     cargo test -p m30-defi-protocols-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod bonus_sandwich;
pub mod ex01_amm_math;
pub mod ex02_amm;
pub mod ex03_lending_math;
pub mod ex04_oracle;
pub mod ex05_lending;
pub mod util;

// One program per .so: `cargo build-sbf --features amm`, etc.
#[cfg(all(target_os = "solana", feature = "amm"))]
mod entry {
    use crate::ex02_amm::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "oracle"))]
mod entry {
    use crate::ex04_oracle::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "lending"))]
mod entry {
    use crate::ex05_lending::process;
    solana_program::entrypoint!(process);
}
