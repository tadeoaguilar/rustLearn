# 26 · Solana Basics

## Overview

Solana runs programs written in Rust, compiled to SBF bytecode, on thousands
of validators that agree on one ledger. What makes it different from the
code in earlier phases isn't the language -- it's the model: programs are
**stateless**, all state lives in **accounts**, every transaction declares up
front which accounts it touches (so non-overlapping transactions run in
parallel), and the runtime enforces strict rules about who may change what.
This module builds wallets, transfers and three small programs on the real
`solana-program` API, and runs them natively in `solsim`.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Wallets | An address is a public key; keypair files; signatures |
| 2 | Transfers | System program instructions, atomic transactions, rent |
| 3 | Hello, World | An entrypoint, logs, return data, owner checks |
| 4 | Borsh | Instruction data and account state as bytes |
| 5 | Notes | PDAs; creating, updating and closing accounts |
| 6 | Vault | CPIs with `invoke` and `invoke_signed` |
| Bonus | Indexing | `getProgramAccounts` with a memcmp filter |

## Key Concepts

### Accounts

```
Account {
    lamports:   u64,        // balance (1 SOL = 10^9 lamports)
    data:       Vec<u8>,    // whatever the owner program stores
    owner:      Pubkey,     // the program that may change data / debit lamports
    executable: bool,       // a program?
}
```

A wallet is an account owned by the **System program** with no data. A
program is an executable account. A program's state -- a counter, a note, a
token balance -- is an account **owned by that program**. Only the owner may
change an account's data or take its lamports; anyone may add lamports.

### Instructions and transactions

```rust
Instruction {
    program_id,                                  // which program runs
    accounts: vec![AccountMeta::new(k, signer)], // every account it may touch, with flags
    data: vec![...],                             // what to do: bytes the program parses
}
```

A transaction is a list of instructions plus signatures. Its instructions run
in order; if one fails, the whole transaction is rolled back.

### Rent

Accounts pay for their storage with a deposit: `Rent::minimum_balance(len)`
lamports (890,880 for a wallet, about 0.007 SOL per extra KB). An account must
hold at least that much -- or nothing, in which case it's deleted. Closing an
account returns the deposit.

### Program Derived Addresses

```rust
let (pda, bump) = Pubkey::find_program_address(&[b"note", author.as_ref(), &id.to_le_bytes()], &program_id);
```

An address derived from seeds and a program id, guaranteed to have no private
key. Clients compute it from the seeds -- no index needed -- and only the
program can sign for it, by passing the seeds to `invoke_signed`.

### Cross-program invocation

A program calls another with `invoke(&instruction, &account_infos)`. It can
pass on the signatures and write access it received, never more. A PDA's
signature comes from `invoke_signed` with its seeds.

### The runtime's rules (enforced by `solsim` too)

| Rule | Error if broken |
|---|---|
| only the owner changes data | `ExternalAccountDataModified` |
| only the owner debits lamports | `ExternalAccountLamportSpend` |
| read-only accounts don't change | `ReadonlyDataModified` / `ReadonlyLamportChange` |
| lamports are neither created nor destroyed | `UnbalancedInstruction` |
| accounts end rent-exempt or empty | `InsufficientFundsForRent` |
| a CPI can't add signers or writers | `PrivilegeEscalation` |

## Common Pitfalls

1. **Trusting accounts because they were passed in** -- check owner, address (PDA), signer, writable
2. **Forgetting the signer check** -- "the author's key is in the accounts" is not "the author signed"
3. **`try_from_slice` on padded account data** -- it rejects trailing bytes; deserialize a prefix
4. **Unchecked arithmetic on balances** -- use `checked_add` / `checked_sub` (release builds here keep overflow checks on too)
5. **Accepting any bump** -- use the canonical one from `find_program_address`
6. **Closing an account by only zeroing lamports** -- also wipe the data and give it back to the System program
7. **Expecting `msg!` output in tests** -- natively it's printed to stdout; on a cluster it's in the transaction logs

## Running This Module

From `06-blockchain-solana/` (Phase 6 is a separate Cargo workspace):

```bash
cargo run  -p m26-solana-basics -- 1                     # your code (1-6, bonus, all)
cargo test -p m26-solana-basics-tests --features mine    # test your code
cargo run  -p m26-solana-basics-solution -- all          # the reference solution
cargo test -p m26-solana-basics-tests                    # 34 tests against the solution
```

Optional, with the Solana toolchain: build the programs for the real VM and
run them in LiteSVM, or deploy them to a local validator --
see [GETTING_STARTED.md](GETTING_STARTED.md).

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (set up an environment, create and fund a
wallet, a "Hello World" program, deploy and interact).

- **No validator, devnet or toolchain is needed.** "Fund a wallet" is
  `Sim::airdrop`; "deploy" is `Sim::add_program`. Deploying to a local
  validator or devnet is an optional section in GETTING_STARTED.md, and the
  `sbf` test feature runs the same programs as bytecode in LiteSVM.
- **Exercises 4-6 go further than the outline** (PDAs, account lifecycle,
  CPIs) because modules 27-30 build on them.
- **Not covered**: Proof of History and consensus (no code to write),
  web3.js (TypeScript), versioned transactions and lookup tables.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[27 · Anchor Framework](../27-anchor-framework/)
