# Getting Started with 29 · NFTs and Tokens

## Quick Start

From **`06-blockchain-solana/`**:

```bash
cargo run -p m29-nfts-tokens-solution -- all
```

creates a token and moves it around (with a frozen account and a fixed
supply refusing transfers and mints), mints an NFT and verifies its
collection, buys two numbered NFTs from a vending machine (and is refused a
third), stakes an NFT for an hour, and transfers a compressed NFT by proof.
The `Instruction: ...` lines are the SPL Token program's own logs.

## What Is Already Here

```
29-nfts-tokens/
├── README.md, exercises.md, GETTING_STARTED.md
├── exercise/src/              # ← YOUR WORKSPACE (package m29-nfts-tokens)
│   ├── util.rs                #   provided (module 28's)
│   ├── ex01_fungible.rs       #   Ex 1  client: SPL Token + ATA
│   ├── ex02_metadata.rs       #   Ex 2  the metadata program + mint_nft (builders given)
│   ├── ex03_vending.rs        #   Ex 3  (builders given)
│   ├── ex04_nft_staking.rs    #   Ex 4  (builders given)
│   └── bonus_compressed.rs    #   bonus
├── solution/                  # ← REFERENCE (m29-nfts-tokens-solution) + ANSWERS.md
└── tests/                     # ← 23 tests (m29-nfts-tokens-tests), + 3 on-chain
```

## The Commands You Need

```bash
cargo run  -p m29-nfts-tokens -- 1
cargo test -p m29-nfts-tokens-tests --features mine ex1_
cargo test -p m29-nfts-tokens-tests --features mine
cargo test -p m29-nfts-tokens-tests                    # the solution: always green
```

Exercises 3 and 4 use Exercise 2's program (and `mint_nft`), so their tests
fail until Exercise 2 passes.

## On the Real Solana VM (Optional)

```bash
./build-sbf.sh solution 29      # metadata.so, vending.so, nft-staking.so
cargo test -p m29-nfts-tokens-tests --features sbf
./build-sbf.sh mine 29
cargo test -p m29-nfts-tokens-tests --features sbf,mine sbf_
```

LiteSVM ships the SPL Token and ATA programs, so the vending machine's four
CPIs run against the real deployed programs.

## If You Get Stuck

SPL Token's error codes (as `Custom(n)`): 1 insufficient funds, 3 mint
mismatch, 4 owner mismatch, 5 fixed supply, 17 account frozen.

1. **`MissingSignature` for the mint** -- a new mint account must sign its own creation: list it among the signers.
2. **`IllegalOwner` creating an ATA idempotently** -- an account exists at that address but isn't the wallet's token account for that mint.
3. **`NotMintAuthority` from your vending machine** -- the metadata CPI needs the machine PDA as mint authority *and* its seeds in `invoke_signed`.
4. **`PrivilegeEscalation` in the vending machine** -- the new mint must be passed writable and signer through to the System program's `create_account`.
5. **`NotInCollection` for a real member** -- did anyone call `verify_collection`?
6. Compare with `solution/src/` -- same file and function names.
