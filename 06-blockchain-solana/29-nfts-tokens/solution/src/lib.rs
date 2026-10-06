//! Module 29 -- NFTs and Tokens. Reference solution.
//!
//! | File                    | Exercise |
//! |-------------------------|----------|
//! | `ex01_fungible.rs`      | 1  a fungible token with the SPL Token program (client side) |
//! | `ex02_metadata.rs`      | 2  NFTs: a metadata program (creators, royalties, verified collections) |
//! | `ex03_vending.rs`       | 3  a vending machine that mints NFTs by CPI into four programs |
//! | `ex04_nft_staking.rs`   | 4  staking NFTs of a verified collection for reward tokens |
//! | `bonus_compressed.rs`   | bonus: compressed NFTs -- Merkle proofs |
//! | `util.rs`               | provided: PDA accounts, token CPIs (module 28's) |

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
