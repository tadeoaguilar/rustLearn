# solsim -- an in-process Solana runtime

Phase 6's tests run Solana programs as ordinary Rust, natively, inside
`solsim`. A program is its entrypoint function; `solsim` is the runtime
around it. No validator, no `cargo build-sbf`, no network: `cargo test` works
with a plain Rust install, and each test runs in milliseconds.

```rust
let mut sim = Sim::new();                                   // System, SPL Token, ATA built in
sim.add_program(my_program::ID, my_program::process);        // "deploy"
let alice = sim.funded_wallet(LAMPORTS_PER_SOL);             // "airdrop"
let meta = sim.process(&[ix1, ix2], &[alice])?;              // a transaction, atomic
let account = sim.account(&key);                             // read state
sim.advance_time(3600);                                      // the clock, for Clock::get()
```

## What It Enforces

The same rules as the real runtime, checked after every program invocation
and around every CPI:

| Rule | Error |
|---|---|
| a meta marked signer must be among the transaction's signers | `MissingSignature` |
| only an account's owner changes its data or debits its lamports | `ExternalAccountDataModified`, `ExternalAccountLamportSpend` |
| read-only accounts don't change | `ReadonlyDataModified`, `ReadonlyLamportChange` |
| the owner changes only by the old owner, on zeroed data | `ModifiedProgramId` |
| lamports are conserved per instruction | `UnbalancedInstruction` |
| a CPI passes on signer/writable privileges, never adds them; `invoke_signed` seeds must derive a PDA of the caller | `PrivilegeEscalation`, `InvalidSeeds` |
| no reentrancy (A -> B -> A), self-recursion allowed; at most 4 nested CPIs | `ReentrancyNotAllowed`, `CallDepthExceeded` |
| a failed CPI fails the whole transaction | the CPI's error |
| accounts end rent-exempt or empty; empty accounts are deleted | `InsufficientFundsForRent` |

`TxError` carries the failing instruction's index, the error and the logs
(`Program X invoke [n]`, `Program X success/failed`, `Program data: ...`).

## How It Works

- **Accounts in memory.** A transaction works on copies; they're committed
  only if every instruction succeeds.
- **Account layout.** Each account gets the memory layout the real runtime's
  serialized input has around it (the original data length before the key,
  a length header and 10 KiB of spare room around the data), so
  `AccountInfo::resize` (realloc) and `original_data_len` work natively.
- **Syscalls.** Off-chain, `solana-program` routes `invoke_signed`,
  `Clock::get()`, `Rent::get()`, `set_return_data`, `sol_log_data` to a global
  `SyscallStubs` object. `solsim` installs one that finds the running
  transaction in a thread-local, so tests run in parallel.
- **Built-in programs.** The System program (create, assign, allocate,
  transfer) is reimplemented in `system.rs`; the **real** SPL Token program
  runs from the `spl-token` crate's processor; the ATA program (`ata.rs`) is
  a small reimplementation at the real address with the real instruction format.

## The `solana-invoke` Patch

Anchor 1.2 makes CPIs through the `solana-invoke` crate, which -- unlike
`solana_program::program::invoke_signed` -- panics off-chain instead of
calling the syscall stubs. `patches/solana-invoke` is a copy of version
0.5.0 (MIT/Apache-2.0) whose off-chain `invoke_signed_unchecked` forwards to
`solana_sysvar::program_stubs::sol_invoke_signed`; the workspace uses it via
`[patch.crates-io]`. The on-chain code path is untouched (and the `sbf`
tests exercise it on the real VM).

## What It Doesn't Do

Signatures (a transaction lists the keys that "signed"), fees, compute
units, the BPF loader and program upgrades, address lookup tables, more
than 10 KiB of account growth per instruction, and parallel scheduling.
Anchor's `emit!` events are a no-op off-chain in Anchor 1.2, so they don't
reach the logs.

## The Real VM (`sbf` feature)

`solsim::sbf::Vm` wraps LiteSVM -- the validator's runtime as a library --
to run the programs `../build-sbf.sh` builds, with real signatures and
compute metering. Each module's tests have a `sbf` feature that uses it.
