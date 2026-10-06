# 29 · NFTs and Tokens

## Overview

Solana doesn't have a contract per token: one program, SPL Token, manages
every mint and every balance, and new tokens are just new accounts. That
makes a token cheap to create and every wallet able to hold any token
without new code. This module uses the real SPL Token program from the
client side, writes the metadata program that turns a mint into an NFT,
and builds two programs on top: a vending machine that mints NFTs, and a
farm that rewards staking them.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Fungible tokens | Mints, ATAs, `_checked` instructions, freeze, fixed supply |
| 2 | Metadata | What makes a mint an NFT; creators, royalties, verified collections |
| 3 | Vending machine | A PDA mint authority; four CPIs in one instruction |
| 4 | NFT staking | Gating on a verified collection; custody; time-based rewards |
| Bonus | Compressed NFTs | Merkle proofs instead of accounts |

## Key Concepts

### Mints and token accounts

```
Mint     { mint_authority, supply, decimals, freeze_authority }          82 bytes
Account  { mint, owner, amount, delegate, state (frozen?), ... }        165 bytes
ATA      = PDA of the ATA program, seeds [wallet, token program, mint]
```

A token account's `owner` field is who controls the tokens -- a wallet, or
a PDA. The *account's* owner (in the runtime sense) is always the token
program.

### An NFT

| Property | How |
|---|---|
| indivisible | `decimals = 0` |
| unique | `supply = 1`, then `mint_authority = None` |
| described | a metadata PDA `["metadata", mint]` (Metaplex in production) |
| part of a collection | `collection.verified`, signed by the collection's authority |

### Authorities

| Authority | Can |
|---|---|
| mint authority | create supply (`None` = fixed forever) |
| freeze authority | freeze/thaw token accounts of the mint |
| token account owner | transfer, burn, approve a delegate, close |
| metadata update authority | change metadata while it's mutable |

A PDA can hold any of them: that's how programs mint (Exercise 3) and pay
rewards (Exercise 4) under their own rules.

## Common Pitfalls

1. **Non-idempotent ATA creation** -- the recipient may already have one; use `CreateIdempotent`
2. **Plain `transfer` with UI amounts** -- multiply by `10^decimals`; prefer `transfer_checked`
3. **Leaving the mint authority set on an "NFT"** -- then it isn't unique
4. **Trusting metadata names or images** -- check the verified collection's mint address
5. **Letting the user hold the mint authority during a mint flow** -- they can skip your rules
6. **Not checking the token account's mint and owner fields** -- a token account of another mint or owner passes an owner check
7. **Forgetting rent for new token accounts** -- someone pays ~0.002 SOL for each

## Running This Module

From `06-blockchain-solana/`:

```bash
cargo run  -p m29-nfts-tokens -- 1                    # your code (1-4, bonus, all)
cargo test -p m29-nfts-tokens-tests --features mine   # test your code
cargo run  -p m29-nfts-tokens-solution -- all         # the reference solution
cargo test -p m29-nfts-tokens-tests                   # 23 tests against the solution
```

Optional: `./build-sbf.sh solution 29` and `--features sbf` run the three
programs as SBF bytecode in LiteSVM (3 tests) -- see
[GETTING_STARTED.md](GETTING_STARTED.md).

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a fungible token, an NFT collection, a
token vending machine, NFT staking).

- **Metaplex isn't used.** Its Token Metadata program is large and deployed,
  not a library; Exercise 2 writes a small program with the same model
  (metadata PDA, creators, verified collections, mutability). The mapping to
  Metaplex is in the README's tables; Candy Machine is Exercise 3's model.
- **Token-2022** (transfer fees, confidential transfers, on-mint metadata)
  isn't covered: `solsim` provides the original SPL Token program only.
- **Compressed NFTs** are the bonus's Merkle maths, without Bubblegum's
  concurrent-tree program or an indexer.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[30 · DeFi Protocols](../30-defi-protocols/)
