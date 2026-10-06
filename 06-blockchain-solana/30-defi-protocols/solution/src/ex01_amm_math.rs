//! Exercise 1: constant-product AMM maths.
//!
//! A pool holds reserves `x` of token A and `y` of token B and keeps
//! `x * y = k` constant across swaps: give it `dx`, it gives back the `dy`
//! that keeps the product (plus a fee that stays in the pool, so `k` grows).
//! Liquidity providers own the pool in proportion to LP *shares*.
//!
//! Integer division rounds down, and every rounding must favour the pool:
//! users receive a little less, pay a little more. Otherwise, repeating a
//! tiny operation in a loop prints money.

/// Fees are in basis points: 30 = 0.30% (Uniswap V2's fee).
pub const BPS: u64 = 10_000;

/// Shares burned on the first deposit, so the share price can never be
/// pushed up to a value where later depositors round down to 0 shares
/// (the "first depositor inflation attack"). Uniswap V2 does the same.
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

/// Integer square root: the largest `r` with `r * r <= n`.
pub fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    // Newton's method, starting above the root.
    let mut x = 1u128 << (n.ilog2() / 2 + 1);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// Swap `amount_in` into a pool with these reserves: how much comes out?
/// The fee is taken from the input. `None` if a reserve is empty or the
/// fee is over 100%.
pub fn swap_out(amount_in: u64, reserve_in: u64, reserve_out: u64, fee_bps: u64) -> Option<u64> {
    if reserve_in == 0 || reserve_out == 0 || fee_bps > BPS {
        return None;
    }
    let in_after_fee = amount_in as u128 * (BPS - fee_bps) as u128;
    let numerator = in_after_fee * reserve_out as u128;
    let denominator = reserve_in as u128 * BPS as u128 + in_after_fee;
    u64::try_from(numerator / denominator).ok()
}

/// The first deposit: `(shares for the depositor, shares locked forever)`.
/// Total shares are `sqrt(a * b)`, so the share price doesn't depend on the
/// ratio the pool starts at. `None` if too small to cover the locked amount.
pub fn initial_shares(amount_a: u64, amount_b: u64) -> Option<(u64, u64)> {
    let total = u64::try_from(isqrt(amount_a as u128 * amount_b as u128)).ok()?;
    let user = total.checked_sub(MINIMUM_LIQUIDITY)?;
    (user > 0).then_some((user, MINIMUM_LIQUIDITY))
}

/// Later deposits: up to `max_a` and `max_b`, in the pool's ratio.
/// Returns `(shares, amount_a, amount_b)`: the shares are rounded down, the
/// amounts taken for them rounded up.
pub fn deposit(
    max_a: u64,
    max_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    supply: u64,
) -> Option<(u64, u64, u64)> {
    if reserve_a == 0 || reserve_b == 0 || supply == 0 {
        return None;
    }
    let (s, ra, rb) = (supply as u128, reserve_a as u128, reserve_b as u128);
    let shares = (max_a as u128 * s / ra).min(max_b as u128 * s / rb);
    if shares == 0 {
        return None;
    }
    let amount_a = (shares * ra).div_ceil(s);
    let amount_b = (shares * rb).div_ceil(s);
    Some((
        u64::try_from(shares).ok()?,
        u64::try_from(amount_a).ok()?,
        u64::try_from(amount_b).ok()?,
    ))
}

/// Burning `shares` of `supply`: the reserves paid out, rounded down.
pub fn withdraw(shares: u64, reserve_a: u64, reserve_b: u64, supply: u64) -> Option<(u64, u64)> {
    if supply == 0 || shares > supply {
        return None;
    }
    let a = shares as u128 * reserve_a as u128 / supply as u128;
    let b = shares as u128 * reserve_b as u128 / supply as u128;
    Some((a as u64, b as u64))
}

/// The marginal price of A in B (how many B one A is worth), as a float --
/// for display only; on-chain maths stays in integers.
pub fn spot_price(reserve_a: u64, reserve_b: u64) -> f64 {
    reserve_b as f64 / reserve_a as f64
}

/// How much worse than the spot price a swap executes, in basis points.
pub fn price_impact_bps(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u64,
) -> Option<u64> {
    let out = swap_out(amount_in, reserve_in, reserve_out, fee_bps)? as u128;
    // At the spot price, amount_in would buy amount_in * reserve_out / reserve_in.
    let ideal = amount_in as u128 * reserve_out as u128 / reserve_in as u128;
    if ideal == 0 {
        return Some(0);
    }
    Some(((ideal - out.min(ideal)) * BPS as u128 / ideal) as u64)
}
