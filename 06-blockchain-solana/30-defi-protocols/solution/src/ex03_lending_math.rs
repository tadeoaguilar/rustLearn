//! Exercise 3: lending maths.
//!
//! A lending market lends out what depositors supply. Borrowers post
//! collateral worth more than they borrow; if prices move and their debt
//! approaches the collateral's value, anyone may *liquidate* them: repay part
//! of the debt and take collateral worth a little more (the bonus) in
//! exchange. Interest accrues continuously through a *borrow index*: the
//! value of one unit borrowed at the start. A borrower stores their debt
//! divided by the index at the time; their debt now is that times today's
//! index.
//!
//! Fixed point: `WAD = 10^18` stands for 1.0; rates and ratios are in
//! basis points (10_000 = 100%).

pub const WAD: u128 = 1_000_000_000_000_000_000;
pub const BPS: u128 = 10_000;
pub const SECONDS_PER_YEAR: u128 = 365 * 24 * 60 * 60;

/// A kinked interest rate curve: cheap money while there's plenty to lend,
/// expensive as the pool runs dry (so borrowers repay and lenders arrive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateModel {
    /// Annual borrow rate at 0% utilization, in bps.
    pub base_bps: u64,
    /// Added between 0% and the kink.
    pub slope1_bps: u64,
    /// Added between the kink and 100%.
    pub slope2_bps: u64,
    /// Utilization at which the steep slope begins, in bps.
    pub kink_bps: u64,
}

/// Borrowed / supplied, in bps (0 if nothing is supplied; capped at 100%).
pub fn utilization_bps(borrowed: u64, supplied: u64) -> u64 {
    if supplied == 0 {
        return 0;
    }
    ((borrowed as u128 * BPS / supplied as u128).min(BPS)) as u64
}

impl RateModel {
    /// The annual borrow rate at `utilization`, in bps.
    pub fn borrow_rate_bps(&self, utilization_bps: u64) -> u64 {
        let u = utilization_bps.min(BPS as u64) as u128;
        let kink = self.kink_bps.max(1) as u128;
        let rate = if u <= kink {
            self.base_bps as u128 + self.slope1_bps as u128 * u / kink
        } else {
            self.base_bps as u128
                + self.slope1_bps as u128
                + self.slope2_bps as u128 * (u - kink) / (BPS - kink).max(1)
        };
        rate as u64
    }
}

/// The index after `seconds` at an annual `rate_bps`, simple interest per
/// accrual (accruing often makes it compound): `index * (1 + rate * t / year)`.
pub fn accrue_index(index: u128, rate_bps: u64, seconds: u64) -> Option<u128> {
    let growth = (rate_bps as u128)
        .checked_mul(seconds as u128)?
        .checked_mul(WAD)?
        / (BPS * SECONDS_PER_YEAR);
    index.checked_mul(WAD.checked_add(growth)?).map(|v| v / WAD)
}

/// What a borrower stores for borrowing `amount` at `index`: rounded *up*,
/// so the protocol never under-records a debt.
pub fn scaled_debt(amount: u64, index: u128) -> u128 {
    (amount as u128 * WAD).div_ceil(index)
}

/// The debt now, from the stored scaled debt: rounded up.
pub fn current_debt(scaled: u128, index: u128) -> Option<u64> {
    u64::try_from((scaled.checked_mul(index)?).div_ceil(WAD)).ok()
}

/// A token amount's value: `amount * price / 10^decimals`, where `price` is
/// the value of one whole token (in any unit, e.g. micro-dollars).
pub fn value(amount: u64, price: u64, decimals: u8) -> u128 {
    amount as u128 * price as u128 / 10u128.pow(decimals as u32)
}

/// How much may be borrowed against `collateral_value` at a loan-to-value of `ltv_bps`.
pub fn max_borrow_value(collateral_value: u128, ltv_bps: u64) -> u128 {
    collateral_value * ltv_bps as u128 / BPS
}

/// `collateral_value * threshold / debt_value` as a WAD: below 1.0 means
/// the position may be liquidated. No debt is infinitely healthy (`u128::MAX`).
pub fn health_factor(
    collateral_value: u128,
    liquidation_threshold_bps: u64,
    debt_value: u128,
) -> u128 {
    if debt_value == 0 {
        return u128::MAX;
    }
    collateral_value
        .saturating_mul(liquidation_threshold_bps as u128)
        .saturating_mul(WAD / BPS)
        / debt_value
}

/// The result of a liquidation, all in token base units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Liquidation {
    /// Debt the liquidator repays.
    pub repay: u64,
    /// Collateral the liquidator receives for it.
    pub seize: u64,
}

/// A liquidator offers to repay up to `max_repay` of `debt` (in the borrow
/// token). At most `close_factor_bps` of the debt can be repaid at once; the
/// liquidator receives collateral worth the repaid value plus `bonus_bps`,
/// but never more than there is (then the repayment shrinks to match).
#[allow(clippy::too_many_arguments)] // each is a separate input of the formula
pub fn liquidate(
    debt: u64,
    collateral: u64,
    max_repay: u64,
    close_factor_bps: u64,
    bonus_bps: u64,
    debt_price: u64,
    debt_decimals: u8,
    collateral_price: u64,
    collateral_decimals: u8,
) -> Liquidation {
    let cap = (debt as u128 * close_factor_bps as u128 / BPS) as u64;
    let mut repay = max_repay.min(cap).min(debt);
    // collateral = repay value * (1 + bonus) / collateral price
    let seize_for = |repay: u64| -> u128 {
        value(repay, debt_price, debt_decimals) * (BPS + bonus_bps as u128) / BPS
            * 10u128.pow(collateral_decimals as u32)
            / collateral_price.max(1) as u128
    };
    let mut seize = seize_for(repay);
    if seize > collateral as u128 {
        // Not enough collateral: repay only what it covers.
        seize = collateral as u128;
        let collateral_value = value(collateral, collateral_price, collateral_decimals);
        let repay_value = collateral_value * BPS / (BPS + bonus_bps as u128);
        repay = (repay_value * 10u128.pow(debt_decimals as u32) / debt_price.max(1) as u128) as u64;
    }
    Liquidation {
        repay,
        seize: seize as u64,
    }
}
