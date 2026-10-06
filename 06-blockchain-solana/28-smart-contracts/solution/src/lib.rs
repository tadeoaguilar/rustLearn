//! Module 28 -- Smart Contracts. Reference solution.
//!
//! Four native programs that hold other people's money, and a security
//! review of a fifth.
//!
//! | File                    | Exercise |
//! |-------------------------|----------|
//! | `ex01_multisig.rs`      | 1  a multisig that executes arbitrary approved instructions |
//! | `ex02_escrow.rs`        | 2  a token escrow: an atomic, trustless swap |
//! | `ex03_staking.rs`       | 3  staking with continuous rewards (reward-per-token accumulator) |
//! | `ex04_security/`        | 4  six classic vulnerabilities: exploit them, then fix them |
//! | `bonus_governance.rs`   | bonus: the multisig changes its own rules through a proposal |
//! | `util.rs`               | provided: PDA accounts, token CPIs |

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
