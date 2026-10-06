# Getting Started with 28 · Smart Contracts

## Quick Start

From **`06-blockchain-solana/`**:

```bash
cargo run -p m28-smart-contracts-solution -- all
```

runs a 2-of-3 multisig paying out after the second approval, a token swap
through an escrow (and a taker failing to redirect the payment), two
stakers sharing rewards 1:3, the duplicate-account exploit draining the
vulnerable bank but not the secure one, and the multisig raising its own
threshold. The SPL Token program's `Instruction: ...` lines are its `msg!`
output, printed natively.

## What Is Already Here

```
28-smart-contracts/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m28-smart-contracts)
│   ├── util.rs                #   provided: PDA accounts, token CPIs
│   ├── ex01_multisig.rs       #   Ex 1  state, instructions, builders given; processor todo!()
│   ├── ex02_escrow.rs         #   Ex 2
│   ├── ex03_staking.rs        #   Ex 3  plus the accrue/settle maths
│   ├── ex04_security/
│   │   ├── mod.rs             #   provided: shared state, instructions, builders
│   │   ├── vulnerable.rs      #   provided: the bank with six bugs
│   │   └── secure.rs          #   Ex 4  a copy of vulnerable.rs -- fix it
│   └── bonus_governance.rs    #   bonus
├── solution/                  # ← REFERENCE (m28-smart-contracts-solution) + ANSWERS.md
└── tests/                     # ← 28 tests (m28-smart-contracts-tests), + 4 on-chain
```

## The Commands You Need

```bash
cargo run  -p m28-smart-contracts -- 2
cargo test -p m28-smart-contracts-tests --features mine
cargo test -p m28-smart-contracts-tests --features mine ex4_bug    # the exploits
cargo test -p m28-smart-contracts-tests                            # the solution: always green
```

The `ex4_bug*` tests fail until you fix `secure.rs`: each runs an exploit
against `vulnerable.rs` (it must work) and then against `secure.rs` (it must
fail). Read them -- they're the attack descriptions.

## On the Real Solana VM (Optional)

```bash
./build-sbf.sh solution 28      # multisig, escrow, staking, bank, bank-secure .so files
cargo test -p m28-smart-contracts-tests --features sbf
./build-sbf.sh mine 28
cargo test -p m28-smart-contracts-tests --features sbf,mine sbf_
```

The on-chain tests run the honest flows (and one refused self-transfer);
the exploits use an attacker program that only exists natively.

## If You Get Stuck

1. **`PrivilegeEscalation` executing a proposal** -- the vault must sign through `invoke_signed` with its seeds, and every account the stored instruction uses must be passed (after the first four).
2. **`IllegalOwner` from `util::token_account`** -- the account isn't a token account (owned by the SPL Token program).
3. **The SPL Token program returns `Custom(1)`** -- insufficient funds; `Custom(3)`: mint mismatch; `Custom(4)`: owner mismatch.
4. **Rewards are 0** -- multiply before you divide (`elapsed * rate * PRECISION / total`).
5. **"borrow failed" during a CPI** -- a `RefMut` of an account's data is alive while you `invoke`; save, drop, then call.
6. **The duplicate-account exploit still works** -- compare the keys of `from` and `to` before loading either.
7. Compare with `solution/src/` -- same file and function names.
