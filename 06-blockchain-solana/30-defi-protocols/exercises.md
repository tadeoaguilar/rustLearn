# Exercises: DeFi Protocols

Decentralized finance is a handful of mechanisms, each a few formulas
wrapped in the account checks of module 28: an **AMM** prices trades with
`x * y = k`; a **lending market** lends against collateral, with interest
set by utilization and **liquidations** keeping it solvent; an **oracle**
tells programs what things are worth. The maths is integer arithmetic where
every rounding direction is a security decision.

| Exercise | Builds | File |
|---|---|---|
| 1 | AMM maths (property-tested) | `ex01_amm_math.rs` |
| 2 | the AMM program | `ex02_amm.rs` |
| 3 | lending maths | `ex03_lending_math.rs` |
| 4 | an oracle and safe price reads | `ex04_oracle.rs` |
| 5 | a lending market | `ex05_lending.rs` |
| bonus | a sandwich attack | `bonus_sandwich.rs` |

**Setup**: as in modules 28-29 -- native programs, SPL tokens, `util.rs`
provided (plus `token_burn`). Builders and state are given; you write the
maths and the processors. Exercise 5 uses Exercises 3 and 4.

---

## Exercise 1: Constant-Product Maths

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Price swaps with `x * y = k` and a fee
- Mint LP shares fairly; defend the first deposit
- Round in the pool's favour, and prove it with property tests

1. `isqrt(n)`: the floor square root of a `u128` (Newton's method).
2. `swap_out(amount_in, reserve_in, reserve_out, fee_bps)`:
   `out = reserve_out * in' / (reserve_in + in')` with `in' = amount_in * (1 - fee)`,
   rounded down; `None` for empty reserves or a fee over 100%.
3. `initial_shares(a, b)`: `sqrt(a * b)` total, `MINIMUM_LIQUIDITY` of it
   locked; `None` if nothing is left for the depositor.
4. `deposit(max_a, max_b, reserves, supply)`: shares rounded down, the
   amounts charged for them rounded up; `withdraw(shares, ...)` rounded down.
5. `price_impact_bps`.

The property tests check that swaps never shrink `k` and that depositing and
withdrawing never returns more than was put in.

**Question**: why lock `MINIMUM_LIQUIDITY` shares at the first deposit?

---

## Exercise 2: The AMM Program

**Difficulty**: Hard
**Time**: 2 hours

**Learning Objectives**:
- Hold two vaults and mint LP tokens from PDAs
- Keep the pool's own accounting of reserves
- Give users slippage limits on every price-dependent instruction

1. **InitPool**: mints in order (`UnorderedMints`), fee below 1000 bps
   (`BadFee`). Create the pool, both vaults (token-owned by the pool) and the
   LP mint (6 decimals, authority: the pool) at their PDAs.
2. **AddLiquidity**: first deposit with `initial_shares`, later ones with
   `deposit`; `SlippageExceeded` below `min_shares`. Update the stored
   reserves and `total_shares` (which includes the locked shares).
3. **RemoveLiquidity**: `SlippageExceeded` below `min_a`/`min_b`; burn the
   user's LP tokens; pay out.
4. **Swap**: `ZeroAmount`, the accounts' mints (`WrongMint`), the vaults are
   the pool's (`WrongPoolAccount`), `SlippageExceeded` below `min_out`.

**Question**: why store the reserves instead of reading the vaults' balances?

---

## Exercise 3: Lending Maths

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Model interest with a kinked utilization curve
- Track debt with a borrow index
- Compute health and liquidation amounts

1. `utilization_bps`; `RateModel::borrow_rate_bps` -- linear to the kink,
   steeper after it.
2. `accrue_index(index, rate_bps, seconds)`; `scaled_debt` and
   `current_debt`, both rounded *up*.
3. `value(amount, price, decimals)`, `max_borrow_value`, `health_factor`
   (a WAD; below 1.0 is liquidatable).
4. `liquidate(...)`: repay at most `close_factor` of the debt, seize
   collateral worth the repayment plus the bonus -- capped by the collateral
   there is.

**Question**: why does the interest rate jump past the kink?

---

## Exercise 4: Price Oracles

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Publish prices as accounts
- Refuse stale, uncertain or forged prices

1. The oracle program: **Create** a feed at `["feed", authority, id]`;
   **Publish** (the authority only, `NotTheAuthority`) sets price,
   confidence and `publish_time = now`.
2. `validate(feed, now, max_age, max_conf_bps)`: `NonPositivePrice`,
   `StalePrice`, `ConfidenceTooWide`.
3. `read_price`: the account must be owned by the oracle program
   (`NotAFeed`), then `validate`.
4. `Price::scaled(decimals)`.

---

## Exercise 5: A Lending Market

**Difficulty**: Hard
**Time**: 3 hours

**Learning Objectives**:
- Combine the index, oracle prices and health checks in one program
- Make every instruction keep the market solvent
- Implement liquidation

`Market::accrue` (bring the index up to date at the utilization's rate) runs
at the start of every instruction. Then:

1. **InitMarket**: `BadParams` unless LTV < threshold < 100%, bonus at
   most 20%, close factor and kink in range; both feeds owned by the oracle
   program; create the market and vaults.
2. **Supply**: add liquidity. **Deposit**: create the obligation on first
   use; add collateral.
3. **Borrow**: `InsufficientLiquidity`; debt after the borrow must be at most
   LTV x collateral value (`ExceedsLtv`), valued with fresh prices from *the
   market's* feeds (`WrongAccount` otherwise).
4. **Repay** (anyone, for anyone): at most the debt; a full repayment clears
   the scaled debt exactly.
5. **Withdraw**: `InsufficientCollateral`; with debt, the rest must still
   cover it at the LTV (`WouldBeUnhealthy`).
6. **Liquidate**: only below health 1.0 (`Healthy`); use
   `ex03::liquidate`; the liquidator pays the repayment, receives the seized
   collateral.

**Questions**:
1. Why are borrows limited by the LTV, but liquidation triggered by a
   higher threshold?
2. What could an attacker do if `Borrow` accepted any price feed account?

---

## Bonus: A Sandwich Attack

**Difficulty**: Easy
**Time**: 30 minutes

In `bonus_sandwich.rs`: simulate a front-run, the victim's swap (which fails
below its `min_out`), and a back-run; `min_out_with_slippage`; and
`best_attack` over candidate sizes. Compare 10%, 3%, 1% and 0.1% slippage.
