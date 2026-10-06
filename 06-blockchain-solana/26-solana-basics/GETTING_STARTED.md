# Getting Started with 26 · Solana Basics

## Quick Start

Phase 6 is its own Cargo workspace. All commands run from
**`06-blockchain-solana/`**:

```bash
cd 06-blockchain-solana
cargo run -p m26-solana-basics-solution -- all
```

prints a wallet and a signature, transfers (including one that fails
atomically and one that breaks rent rules), the hello program's return data,
a note's life from creation to deletion, and the vault's CPIs in the logs.
The first build takes a few minutes (the Solana crates).

## What Is Already Here

```
06-blockchain-solana/
├── Cargo.toml, build-sbf.sh       # the phase workspace; optional SBF builds
├── sim/                           # solsim: the in-process runtime the tests use
└── 26-solana-basics/
    ├── README.md, exercises.md, GETTING_STARTED.md
    ├── exercise/src/              # ← YOUR WORKSPACE (package m26-solana-basics)
    │   ├── ex01_wallet.rs         #   Ex 1  keypairs, signatures, SOL amounts   (client)
    │   ├── ex02_transfers.rs      #   Ex 2  transfers, rent                     (client)
    │   ├── ex03_hello.rs          #   Ex 3  the hello program                   (on-chain)
    │   ├── ex04_notes_state.rs    #   Ex 4  Borsh instructions and state, PDAs  (both)
    │   ├── ex05_notes.rs          #   Ex 5  the notes program                   (on-chain)
    │   ├── ex06_vault.rs          #   Ex 6  the vault program: CPIs             (on-chain)
    │   └── bonus_client.rs        #   bonus: reading program accounts           (client)
    ├── solution/                  # ← REFERENCE (m26-solana-basics-solution) + ANSWERS.md
    └── tests/                     # ← 34 tests (m26-solana-basics-tests), + 3 on-chain
```

"Client" files only exist off-chain (`#[cfg(not(target_os = "solana"))]` in
`lib.rs`); "on-chain" ones are what `cargo build-sbf` compiles.

## The Commands You Need

```bash
cargo run  -p m26-solana-basics -- 3                     # your code: 1-6, bonus, all
cargo test -p m26-solana-basics-tests --features mine
cargo test -p m26-solana-basics-tests --features mine ex5_
cargo test -p m26-solana-basics-tests                    # the solution: always green
```

To see a failing transaction's logs, print the error: `TxError`'s `Display`
lists them (`println!("{err}")`).

## On the Real Solana VM (Optional)

The tests above run your programs as native Rust. To compile them to SBF
bytecode and run them on the real VM (LiteSVM: the validator's runtime as a
library, with real signatures and compute limits), install the Solana
toolchain (https://solana.com/docs/intro/installation), then:

```bash
./build-sbf.sh                    # the solutions -> target/deploy/solution/{hello,notes,vault}.so
./build-sbf.sh mine 26            # your crate    -> target/deploy/mine/...
cargo test -p m26-solana-basics-tests --features sbf             # 3 tests on the VM, solution
cargo test -p m26-solana-basics-tests --features sbf,mine sbf_   # ...your programs
```

The first `build-sbf` downloads the platform tools (~1 minute). Each crate
holds three programs and a `.so` has one entrypoint, so `build-sbf.sh`
builds the crate once per program with `--features hello|notes|vault`.

### On a Local Validator (Optional, Not Run in This Repository's Checks)

```bash
solana-test-validator --reset                  # terminal 1
solana config set --url localhost              # terminal 2
solana-keygen new --no-bip39-passphrase        # if you have no ~/.config/solana/id.json
solana airdrop 10
solana program deploy target/deploy/solution/hello.so
```

`deploy` prints the program's address (a new keypair, not the `declare_id!`
in the source -- the private keys for those aren't in this repository).
`solana program show <address>` shows it on-chain. This module has no RPC
client to call it; the `sbf` tests are the supported way to run the bytecode.

## If You Get Stuck

1. **`InsufficientFundsForRent`** -- an account ended with lamports, but fewer than `minimum_balance(len)`.
2. **`PrivilegeEscalation`** -- a CPI marked an account signer/writable that the caller didn't have as signer/writable, or the PDA seeds don't match.
3. **`ExternalAccountDataModified`** -- your program wrote to an account it doesn't own.
4. **`AccountAlreadyInUse` (`Custom(0)` from the System program)** -- creating an account that exists.
5. **`InvalidAccountData` from `Note::read`** -- `try_from_slice` rejects the padding; deserialize from `&mut &data[..]`.
6. **"borrow failed"** -- a `RefMut` of an account's data/lamports is still alive when you borrow again or `invoke`; scope it.
7. Compare with `solution/src/` -- same file and function names.
