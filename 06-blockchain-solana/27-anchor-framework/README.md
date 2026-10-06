# 27 · Anchor Framework

## Overview

Anchor is to Solana programs roughly what axum is to HTTP servers: you
declare what an instruction expects -- which accounts, with which
properties -- and it generates the parsing, validation, (de)serialization and
dispatch. Most Solana programs in production are written with it. This
module builds one Anchor program with three features (a counter, polls, a
profile registry), a typed client for testing it, and an account that grows
with `realloc`.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Counter | `init`, PDA `seeds`/`bump`, `has_one`, custom errors, `close` |
| 2 | Voting | `#[instruction]` args in seeds, `InitSpace`/`max_len`, the Clock, once-per-user receipts |
| 3 | Validation | Constraints as an authorization model; type confusion |
| 4 | Client | Generated `accounts::`/`instruction::` structs, safe fetching, error codes |
| Bonus | `realloc` | Growing and shrinking an account, rent included |

## Key Concepts

### A program

```rust
declare_id!("DB6x...");                    // the program's address

#[program]
pub mod anchor_lab {
    pub fn increment(ctx: Context<UpdateCounter>, by: u64) -> Result<()> {
        ctx.accounts.increment(by)          // runs only if every constraint passed
    }
}

#[account]                                 // 8-byte discriminator + Borsh
#[derive(InitSpace)]                       // Counter::INIT_SPACE, for `space =`
pub struct Counter { pub authority: Pubkey, pub count: u64, pub bump: u8 }
```

An instruction's data is an 8-byte discriminator (`sha256("global:increment")`)
followed by its Borsh-encoded arguments; Anchor dispatches on it.

### Account types

| Type | Anchor checks |
|---|---|
| `Signer<'info>` | signed the transaction |
| `Account<'info, T>` | owned by this program, discriminator is `T`'s; deserialized on entry, written back on exit if `mut` |
| `Program<'info, System>` | is that program |
| `SystemAccount<'info>` | owned by the System program |
| `UncheckedAccount<'info>` | nothing (needs a `/// CHECK:` comment) |

### Constraints

| Constraint | Meaning |
|---|---|
| `mut` | writable (and saved after the handler) |
| `init, payer = p, space = n` | create it (CPI to System), `p` pays the rent |
| `seeds = [...], bump` | address is the PDA (`bump` alone: find it; `bump = x.bump`: use the stored one) |
| `has_one = f` | `account.f == f.key()` |
| `constraint = expr @ Err` | any boolean, with your error |
| `close = r` | after the handler: lamports to `r`, account wiped |
| `realloc = n, realloc::payer = p, realloc::zero = false` | resize, adjusting rent |

### Error codes

| Range | Source |
|---|---|
| 100-1999 | instruction errors (unknown discriminator, ...) |
| 2000-2999 | constraint violations (`ConstraintSeeds` 2006, `ConstraintHasOne` 2001) |
| 3000-3999 | account errors (`AccountDiscriminatorMismatch` 3002, `AccountNotSigner` 3010) |
| 6000+ | your `#[error_code]` enum, in order |

## Common Pitfalls

1. **Forgetting `mut`** -- the handler's changes are silently not saved
2. **`init` without `seeds`** -- then the new account must sign, i.e. be a keypair
3. **`space` without the 8-byte discriminator** -- `8 + T::INIT_SPACE`
4. **`bump` instead of `bump = x.bump` on existing accounts** -- re-searches every time (compute)
5. **`#[instruction(...)]` out of order** -- it lists the handler's arguments from the first, in order
6. **Strings and `Vec`s without `#[max_len]`** -- `InitSpace` can't size them
7. **`init_if_needed`** -- convenient, and a reinitialization bug waiting to happen; avoid

## Running This Module

From `06-blockchain-solana/`:

```bash
cargo run  -p m27-anchor-framework -- 1                    # your code (1-4, bonus, all)
cargo test -p m27-anchor-framework-tests --features mine   # test your code
cargo run  -p m27-anchor-framework-solution -- all         # the reference solution
cargo test -p m27-anchor-framework-tests                   # 23 tests against the solution
```

Optional: `./build-sbf.sh` and `--features sbf` run the program as SBF
bytecode in LiteSVM (3 tests) -- see [GETTING_STARTED.md](GETTING_STARTED.md).

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a counter, a voting application, account
validation, comprehensive tests).

- **The `anchor` CLI isn't needed.** The program is a normal crate; tests
  call its generated `entry` function in `solsim`. `anchor build` / `anchor
  test` / IDL generation are optional and haven't been run in this
  repository's checks.
- **CPIs from Anchor reach `solsim` through a patched `solana-invoke`**
  (`../sim/patches/`): off-chain, the upstream crate panics on a CPI. The
  on-chain code path is unchanged, and the `sbf` tests run the unpatched
  behaviour on the real VM.
- **Events (`emit!`) aren't tested**: off-chain, Anchor 1.2 compiles
  `sol_log_data` to a no-op, so they only appear on the VM.
- **"Generate client SDKs"** is Exercise 4's hand-written Rust client; the
  TypeScript client generated from the IDL is out of scope.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[28 · Smart Contracts](../28-smart-contracts/)
