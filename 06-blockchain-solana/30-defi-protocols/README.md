# 30 · DeFi Protocols

## Overview

The protocols holding most of DeFi's value are small programs with
carefully chosen arithmetic: a constant-product AMM is one formula and some
share accounting; a lending market is an interest index, a health check and
a liquidation rule. This module builds both, the oracle they depend on, and
property tests that pin down the invariants -- `k` never shrinks, nobody
gets out more than they put in -- because these are the programs where a
rounding error is an exploit.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | AMM maths | `x * y = k`, fees, LP shares, rounding, property tests |
| 2 | AMM program | Vaults, LP mint, stored reserves, slippage limits |
| 3 | Lending maths | Kinked rates, the borrow index, health, liquidation |
| 4 | Oracles | Staleness, confidence, provenance |
| 5 | Lending market | Deposit, borrow, repay, withdraw, liquidate |
| Bonus | MEV | A sandwich attack, and what slippage limits buy |

## Key Concepts

### The constant-product curve

```
out = reserve_out * in' / (reserve_in + in')      in' = in * (1 - fee)
```

The price is the ratio of reserves; a trade moves it (price impact). The
fee stays in the pool, so `k` grows and LP shares appreciate.

### Rounding

| Operation | Round | So that |
|---|---|---|
| swap output | down | the pool never pays too much |
| shares minted | down | depositors never get extra |
| amounts charged for shares | up | depositors never pay too little |
| withdrawal amounts | down | |
| scaled debt recorded, current debt | up | debt is never forgiven by rounding |

### The borrow index

```
index(t + dt) = index(t) * (1 + rate(utilization) * dt / year)
debt         = scaled_debt * index          (scaled_debt = amount / index_at_borrow)
```

One number per market accrues interest for every borrower at once.

### Health

```
health = collateral_value * liquidation_threshold / debt_value      (< 1: liquidatable)
```

Borrowing is capped at `collateral_value * LTV` (LTV < threshold). A
liquidator repays up to the close factor of the debt and receives that
value plus a bonus in collateral.

### Oracle hygiene

Check the feed is *the configured account* (and owned by the oracle
program), the price is recent, and the confidence interval is narrow.

## Common Pitfalls

1. **Rounding in the user's favour** -- loopable for profit
2. **No slippage parameter** -- every swap is a sandwich opportunity
3. **Prices from vault balances** -- donations move them
4. **Trusting any price account** -- check the address, not just the layout
5. **Stale prices** -- liquidations on old prices transfer value to whoever notices
6. **Interest computed from the last action only** -- accrue before every state change
7. **Multiplying after dividing** -- precision lost; overflow checks off

## Running This Module

From `06-blockchain-solana/`:

```bash
cargo run  -p m30-defi-protocols -- 1                    # your code (1-5, bonus, all)
cargo test -p m30-defi-protocols-tests --features mine   # test your code
cargo run  -p m30-defi-protocols-solution -- all         # the reference solution
cargo test -p m30-defi-protocols-tests                   # 28 tests against the solution
```

Optional: `./build-sbf.sh solution 30` and `--features sbf` run the AMM,
oracle and lending programs as SBF bytecode in LiteSVM (2 tests) -- see
[GETTING_STARTED.md](GETTING_STARTED.md).

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a simple AMM, a lending protocol, a
staking program, price oracles).

- **Staking** was built in module 28 (Exercise 3), so it isn't repeated here.
- **Pyth and Switchboard aren't used**: their feeds are accounts on real
  clusters. Exercise 4's oracle has the same model (price, confidence,
  exponent, publish time) and the same checks a Pyth consumer should make.
- **Suppliers don't receive interest-bearing tokens** in Exercise 5 (real
  markets mint them; the share maths is Exercise 1's LP shares again). Flash loans and governance aren't covered.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

Phase 7: [31 · CLI Tools](../../07-systems-programming/31-cli-tools/)
