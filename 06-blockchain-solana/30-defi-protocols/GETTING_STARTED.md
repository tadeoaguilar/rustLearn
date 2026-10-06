# Getting Started with 30 · DeFi Protocols

## Quick Start

From **`06-blockchain-solana/`**:

```bash
cargo run -p m30-defi-protocols-solution -- all
```

prints price impact for growing swaps, an AMM refusing a swap past its
slippage limit, the interest rate curve, a borrower refused beyond the LTV
and on a stale price, a liquidation after SOL drops 20%, and how profitable
a sandwich attack is at different slippage settings.

## What Is Already Here

```
30-defi-protocols/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m30-defi-protocols)
│   ├── util.rs                #   provided (module 28's + token_burn)
│   ├── ex01_amm_math.rs       #   Ex 1  pure maths
│   ├── ex02_amm.rs            #   Ex 2  program (builders given)
│   ├── ex03_lending_math.rs   #   Ex 3  pure maths
│   ├── ex04_oracle.rs         #   Ex 4  program + consumer checks (builders given)
│   ├── ex05_lending.rs        #   Ex 5  program (builders given)
│   └── bonus_sandwich.rs      #   bonus
├── solution/                  # ← REFERENCE (m30-defi-protocols-solution) + ANSWERS.md
└── tests/                     # ← 28 tests incl. property tests (m30-defi-protocols-tests), + 2 on-chain
```

## The Commands You Need

```bash
cargo run  -p m30-defi-protocols -- 1
cargo test -p m30-defi-protocols-tests --features mine ex1_
cargo test -p m30-defi-protocols-tests --features mine
cargo test -p m30-defi-protocols-tests                    # the solution: always green
```

The property tests (`ex1_isqrt_is_the_floor_root`, `ex1_swaps_never_shrink_k`,
`ex1_deposit_then_withdraw_never_profits`) try hundreds of random inputs;
when one fails, proptest prints the smallest failing case.

## On the Real Solana VM (Optional)

```bash
./build-sbf.sh solution 30      # amm.so, oracle.so, lending.so
cargo test -p m30-defi-protocols-tests --features sbf
./build-sbf.sh mine 30
cargo test -p m30-defi-protocols-tests --features sbf,mine sbf_
```

## If You Get Stuck

1. **Off-by-one in `swap_out`** -- multiply everything first, divide once at the end, in `u128`.
2. **`deposit` returns too many shares** -- take the *minimum* over both tokens.
3. **The second LP gets the wrong share count** -- `total_shares` includes the locked `MINIMUM_LIQUIDITY`.
4. **`StalePrice` in your lending tests** -- the tests republish prices after moving the clock; your `Borrow` must read the feeds with `read_price`, not cache them.
5. **Liquidation numbers off by a few units** -- compute values in `u128` and scale prices with `Price::scaled(6)` before multiplying.
6. **Debt doesn't reach exactly 0 after repaying** -- a full repayment must clear `debt_scaled` itself, not subtract a rounded amount.
7. Compare with `solution/src/` -- same file and function names.
