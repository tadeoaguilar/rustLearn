//! Module 30 -- DeFi Protocols. Reference solution.
//!
//! | File                     | Exercise |
//! |--------------------------|----------|
//! | `ex01_amm_math.rs`       | 1  constant-product maths: swaps, LP shares, rounding |
//! | `ex02_amm.rs`            | 2  the AMM program: pools, vaults, LP tokens, slippage limits |
//! | `ex03_lending_math.rs`   | 3  interest rate curves, the borrow index, health, liquidation |
//! | `ex04_oracle.rs`         | 4  a price oracle and safe price reads |
//! | `ex05_lending.rs`        | 5  a lending market: deposit, borrow, repay, withdraw, liquidate |
//! | `bonus_sandwich.rs`      | bonus: a sandwich attack and slippage limits |
//! | `util.rs`                | provided: PDA accounts, token CPIs (module 28's + burn) |

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
