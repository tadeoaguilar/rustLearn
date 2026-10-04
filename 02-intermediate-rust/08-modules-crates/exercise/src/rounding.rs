//! Rounding helpers for `pricing`.

/// `numerator / denominator`, rounded half away from zero, in integer
/// arithmetic. `round_half_up(1999 * 15, 100)` is 15% of $19.99 in cents: 300.
///
/// `pub(crate)`: callable from anywhere in this crate, invisible outside it.
/// The module is private too, so `pub` would have the same reach today -- but
/// `pub(crate)` says what we *mean*, and stays correct if someone later makes
/// the module public.
pub(crate) fn round_half_up(numerator: i64, denominator: i64) -> i64 {
    todo!("Exercise 2")
}
