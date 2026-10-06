# Getting Started with Phase 6 · Blockchain & Solana

Phase 6 is a **separate Cargo workspace** in this directory: the Solana
crates are a large dependency tree, and Anchor needs Rust 1.89. Run every
command from here:

```bash
cd 06-blockchain-solana
cargo test                                             # all five modules + solsim, against the solutions
cargo run  -p m26-solana-basics-solution -- all        # a module's demo
cargo test -p m26-solana-basics-tests --features mine  # your code
```

The first build downloads and compiles the Solana crates (a few minutes).

## Two Ways to Run a Program

| | Native (default) | On the Solana VM (optional) |
|---|---|---|
| needs | Rust 1.89+ | + the Solana toolchain (`cargo build-sbf`) |
| runs | your program as native Rust in `solsim` | the SBF bytecode in LiteSVM |
| speed | milliseconds per test | a build first, then fast |
| use for | developing, every exercise's tests | checking it really works on-chain |

The native track is what every module is built around; see
[sim/README.md](sim/README.md) for what `solsim` checks. The VM track:

```bash
# install: https://solana.com/docs/intro/installation  (provides cargo build-sbf)
./build-sbf.sh                       # every solution program -> target/deploy/solution/*.so
./build-sbf.sh mine                  # your exercise crates  -> target/deploy/mine/*.so
./build-sbf.sh mine 28               # one module
cargo test -p m28-smart-contracts-tests --features sbf        # the solution's programs on the VM
cargo test -p m28-smart-contracts-tests --features sbf,mine   # yours
```

A `.so` has one entrypoint, and a module crate holds several programs, so
each program has a Cargo feature that switches its `entrypoint!` on;
`build-sbf.sh` builds the crate once per feature (`cargo build-sbf --features
escrow -- --lib`) and names the output after the program.

## Modules

| Module | Programs | Tests (+ on-chain) |
|---|---|---|
| [26 · Solana Basics](26-solana-basics/) | hello, notes, vault | 34 (+3) |
| [27 · Anchor Framework](27-anchor-framework/) | anchor_lab (Anchor) | 23 (+3) |
| [28 · Smart Contracts](28-smart-contracts/) | multisig, escrow, staking, bank, bank-secure | 28 (+4) |
| [29 · NFTs and Tokens](29-nfts-tokens/) | metadata, vending, nft-staking | 23 (+3) |
| [30 · DeFi Protocols](30-defi-protocols/) | amm, oracle, lending | 28 (+2) |
| [sim](sim/) | the runtime | 13 |

## Optional: a Local Validator

To deploy to `solana-test-validator` or devnet, see module 26's
[GETTING_STARTED.md](26-solana-basics/GETTING_STARTED.md#on-a-local-validator-optional-not-run-in-this-repositorys-checks).
The programs' `declare_id!` addresses are random; their private keys aren't
in this repository, so a deployed copy gets a new address.

## Troubleshooting

- **`rustc 1.89 or newer is required`** -- `rustup update stable`.
- **`cargo build-sbf: command not found`** -- the toolchain is optional; the native tests don't need it.
- **`.so is missing`** in an `sbf` test -- run `./build-sbf.sh` (or `./build-sbf.sh mine`) first.
- **`warning: patch solana-invoke was not used`** -- harmless when building a crate that doesn't use Anchor.
- **Lots of `Instruction: Transfer` lines** -- the SPL Token program's `msg!` logs, printed to stdout natively.
