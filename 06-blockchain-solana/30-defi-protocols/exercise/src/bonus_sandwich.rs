//! Bonus: why `min_out` matters -- a sandwich attack, in numbers.
//!
//! Transactions wait in public before they're executed. An attacker who sees
//! a victim's swap of A for B can put a swap of their own just before it
//! (buying B, which pushes B's price up), let the victim buy at the worse
//! price, and sell the B back just after -- pocketing the difference. The
//! victim's only defence is the slippage limit: if the front-run would push
//! the victim's output below `min_out`, the victim's swap fails, and the
//! attacker is left holding B bought at a premium.
//!
//! All amounts are in base units; the pool is `ex01_amm_math`'s.

use crate::ex01_amm_math::swap_out;

/// The attack's outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sandwich {
    /// What the victim received.
    pub victim_out: u64,
    /// The attacker's profit in A (what they got back minus `attacker_in`).
    pub profit: i128,
}

/// Simulate: attacker swaps `attacker_in` A->B, victim swaps `victim_in`
/// A->B with `victim_min_out`, attacker swaps all their B back. `None` if
/// the victim's swap would fail its limit (the sandwich doesn't happen).
pub fn sandwich(
    reserve_a: u64,
    reserve_b: u64,
    fee_bps: u64,
    victim_in: u64,
    victim_min_out: u64,
    attacker_in: u64,
) -> Option<Sandwich> {
    todo!("Bonus")
}

/// The victim's `min_out` for a slippage tolerance: the output at today's
/// price, less `slippage_bps`.
pub fn min_out_with_slippage(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u64,
    slippage_bps: u64,
) -> u64 {
    todo!("Bonus")
}

/// The most profitable attack size among `candidates` (largest profit
/// first), or `None` if no candidate makes money.
pub fn best_attack(
    reserve_a: u64,
    reserve_b: u64,
    fee_bps: u64,
    victim_in: u64,
    victim_min_out: u64,
    candidates: &[u64],
) -> Option<(u64, Sandwich)> {
    todo!("Bonus")
}
