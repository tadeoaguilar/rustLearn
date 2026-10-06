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
    todo!("Exercise 3")
}

impl RateModel {
    /// The annual borrow rate at `utilization`, in bps.
    pub fn borrow_rate_bps(&self, utilization_bps: u64) -> u64 {
        todo!("Exercise 3")
    }
}

/// The index after `seconds` at an annual `rate_bps`, simple interest per
/// accrual (accruing often makes it compound): `index * (1 + rate * t / year)`.
pub fn accrue_index(index: u128, rate_bps: u64, seconds: u64) -> Option<u128> {
    todo!("Exercise 3")
}

/// What a borrower stores for borrowing `amount` at `index`: rounded *up*,
/// so the protocol never under-records a debt.
pub fn scaled_debt(amount: u64, index: u128) -> u128 {
    todo!("Exercise 3")
}

/// The debt now, from the stored scaled debt: rounded up.
pub fn current_debt(scaled: u128, index: u128) -> Option<u64> {
    todo!("Exercise 3")
}

/// A token amount's value: `amount * price / 10^decimals`, where `price` is
/// the value of one whole token (in any unit, e.g. micro-dollars).
pub fn value(amount: u64, price: u64, decimals: u8) -> u128 {
    todo!("Exercise 3")
}

/// How much may be borrowed against `collateral_value` at a loan-to-value of `ltv_bps`.
pub fn max_borrow_value(collateral_value: u128, ltv_bps: u64) -> u128 {
    todo!("Exercise 3")
}

/// `collateral_value * threshold / debt_value` as a WAD: below 1.0 means
/// the position may be liquidated. No debt is infinitely healthy (`u128::MAX`).
pub fn health_factor(
    collateral_value: u128,
    liquidation_threshold_bps: u64,
    debt_value: u128,
) -> u128 {
    todo!("Exercise 3")
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
    todo!("Exercise 3")
}
