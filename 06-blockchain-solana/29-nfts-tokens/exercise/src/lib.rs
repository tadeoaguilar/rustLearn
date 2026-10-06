//! Module 29 -- 29-nfts-tokens. YOUR WORKSPACE.
//!
//! Each `exNN_*.rs` file has the signatures the tests expect, with `todo!()`
//! bodies. Replace each `todo!()` with your implementation, then:
//!
//!     cargo run  -p m29-nfts-tokens -- <exercise number>
//!     cargo test -p m29-nfts-tokens-tests --features mine
//!
//! Stuck? Compare with ../solution/src -- same file and function names.

// Remove this line once you have started: it silences "unused" warnings
// while everything is still todo!().
#![allow(unused)]

pub mod bonus_compressed;
#[cfg(not(target_os = "solana"))]
pub mod ex01_fungible;
pub mod ex02_metadata;
pub mod ex03_vending;
pub mod ex04_nft_staking;
pub mod util;

// One program per .so: `cargo build-sbf --features vending`, etc.
#[cfg(all(target_os = "solana", feature = "metadata"))]
mod entry {
    use crate::ex02_metadata::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "vending"))]
mod entry {
    use crate::ex03_vending::process;
    solana_program::entrypoint!(process);
}
#[cfg(all(target_os = "solana", feature = "nft-staking"))]
mod entry {
    use crate::ex04_nft_staking::process;
    solana_program::entrypoint!(process);
}
