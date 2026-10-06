# Getting Started with 27 · Anchor Framework

## Quick Start

From **`06-blockchain-solana/`**:

```bash
cargo run -p m27-anchor-framework-solution -- all
```

shows Anchor's logs and errors as each feature runs: a counter that refuses
to underflow and refuses other users (`ConstraintSeeds`), a poll that
refuses a second vote and late votes, a registry that refuses non-admins and
type confusion, and a profile account growing and shrinking with `realloc`.

## What Is Already Here

```
27-anchor-framework/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m27-anchor-framework)
│   ├── lib.rs                 #   provided: declare_id!, #[program] routing every instruction
│   ├── errors.rs              #   provided: LabError (codes 6000+)
│   ├── ex01_counter.rs        #   Ex 1  constraints (TODO) + handlers (todo!())
│   ├── ex02_voting.rs         #   Ex 2
│   ├── ex03_registry.rs       #   Ex 3
│   ├── ex04_client.rs         #   Ex 4  client side only
│   └── bonus_realloc.rs       #   bonus
├── solution/                  # ← REFERENCE (m27-anchor-framework-solution) + ANSWERS.md
└── tests/                     # ← 23 tests (m27-anchor-framework-tests), + 3 on-chain
```

The state structs (`Counter`, `Poll`, ...) are given: tests decode them, so
their layout is fixed. In each `#[derive(Accounts)]` struct, a
`// TODO: constraints` marks every field that needs an `#[account(...)]`.

## The Commands You Need

```bash
cargo run  -p m27-anchor-framework -- 1
cargo test -p m27-anchor-framework-tests --features mine
cargo test -p m27-anchor-framework-tests --features mine ex2_
cargo test -p m27-anchor-framework-tests                    # the solution: always green
```

A failing test prints the transaction's error and logs; Anchor's own
messages (`AnchorError caused by account: counter. Error Code:
ConstraintSeeds ...`) are printed to stdout -- run with `-- --nocapture`
to see them.

## On the Real Solana VM (Optional)

```bash
./build-sbf.sh solution 27                 # -> target/deploy/solution/anchor_lab.so
cargo test -p m27-anchor-framework-tests --features sbf
./build-sbf.sh mine 27
cargo test -p m27-anchor-framework-tests --features sbf,mine sbf_
```

`build-sbf.sh` runs `cargo build-sbf` (Solana toolchain required; see
module 26's GETTING_STARTED). `anchor build` would do the same plus an IDL,
but needs an `Anchor.toml` workspace, which this repository doesn't set up.

## If You Get Stuck

1. **Changes don't stick** -- the account needs `mut`.
2. **`ctx.bumps.counter` doesn't exist** -- the field is generated only for accounts with `seeds` + `bump`.
3. **`ConstraintSeeds` (2006)** -- the seeds in the constraint don't match the client's; compare byte by byte (`poll_id.to_le_bytes()`).
4. **`AccountDidNotDeserialize` / `AccountNotInitialized`** -- the account doesn't exist yet: missing `init`?
5. **Unknown `poll_id` in seeds** -- add `#[instruction(poll_id: u64)]` (arguments in order, from the first).
6. **`InstructionDidNotDeserialize` (102)** -- the handler's arguments changed but the test's data didn't; keep the provided signatures.
7. Compare with `solution/src/` -- same file and function names.
