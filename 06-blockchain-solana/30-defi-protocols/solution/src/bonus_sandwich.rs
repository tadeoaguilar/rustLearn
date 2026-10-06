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
    let (mut ra, mut rb) = (reserve_a, reserve_b);
    // 1. front-run
    let attacker_b = swap_out(attacker_in, ra, rb, fee_bps)?;
    ra = ra.checked_add(attacker_in)?;
    rb = rb.checked_sub(attacker_b)?;
    // 2. the victim, at the worse price
    let victim_out = swap_out(victim_in, ra, rb, fee_bps)?;
    if victim_out < victim_min_out {
        return None;
    }
    ra = ra.checked_add(victim_in)?;
    rb = rb.checked_sub(victim_out)?;
    // 3. back-run: sell the B
    let attacker_a = swap_out(attacker_b, rb, ra, fee_bps)?;
    Some(Sandwich {
        victim_out,
        profit: attacker_a as i128 - attacker_in as i128,
    })
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
    let expected = swap_out(amount_in, reserve_in, reserve_out, fee_bps).unwrap_or(0) as u128;
    (expected * (10_000 - slippage_bps.min(10_000)) as u128 / 10_000) as u64
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
    candidates
        .iter()
        .filter_map(|&size| {
            sandwich(
                reserve_a,
                reserve_b,
                fee_bps,
                victim_in,
                victim_min_out,
                size,
            )
            .map(|s| (size, s))
        })
        .filter(|(_, s)| s.profit > 0)
        .max_by_key(|(_, s)| s.profit)
}
