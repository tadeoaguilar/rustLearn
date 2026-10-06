//! Module 26 -- Solana Basics. Reference solution.
//!
//! Accounts, lamports and rent, transactions and instructions, programs,
//! PDAs and cross-program invocations -- on a real Solana API
//! (`solana-program`), run natively by `solsim`.
//!
//! | File                | Exercise | Runs |
//! |---------------------|----------|------|
//! | `ex01_wallet.rs`    | 1  keypairs, keypair files, signatures, SOL amounts | client |
//! | `ex02_transfers.rs` | 2  funding, transfers, rent, atomic transactions | client |
//! | `ex03_hello.rs`     | 3  a "Hello World" program: logs, return data, an owned account | on-chain |
//! | `ex04_notes_state.rs` | 4  instruction data and account state with Borsh | both |
//! | `ex05_notes.rs`     | 5  a notes program: PDAs, create/update/close accounts | on-chain |
//! | `ex06_vault.rs`     | 6  a vault program: CPIs with `invoke` and `invoke_signed` | on-chain |
//! | `bonus_client.rs`   | bonus: reading program accounts like an indexer | client |

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
