# 28 · Smart Contracts

## Overview

A Solana program that holds value is a public API with money behind it:
anyone can call any instruction with any accounts, in any order, combined
with any other instructions in one transaction. This module builds four
programs of the kind that hold real funds on mainnet -- a multisig, an
escrow, a staking pool -- and then attacks a deliberately vulnerable bank
six ways, so that the checks in the first three make sense.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | Multisig | Storing and executing arbitrary instructions; checks-effects-interactions |
| 2 | Escrow | PDA-owned token accounts; atomic swaps; binding accounts to commitments |
| 3 | Staking | The reward-per-token accumulator; scaled integers and rounding |
| 4 | Security review | Six vulnerabilities: exploit, then fix |
| Bonus | Governance | A program changing its own parameters through a self-CPI |

## Key Concepts

### The validation checklist

For every account an instruction receives, ask:

| Check | Why |
|---|---|
| **signer** | is this action authorised by this key? |
| **owner** | did *my program* write these bytes? (else anyone could forge them) |
| **type** | is it the *kind* of account I expect? (tag / discriminator) |
| **address** | is it *the* account (PDA from known seeds), not just *an* account? |
| **relations** | does it belong to the others (`escrow.maker == maker`)? |
| **writable** | will my change persist / am I allowed to change it? |
| **program ids** | is the program I'm about to CPI into the one I think? |

And for every number: checked arithmetic, and the direction of rounding
(always in the protocol's favour).

### Checks, effects, interactions

```rust
validate(...)?;                          // checks
proposal.executed = true; save(...)?;    // effects: my state, final
invoke_signed(&stored_ix, ...)?;         // interactions: someone else's code
// don't write state loaded before the CPI after it
```

### PDA authority

A PDA can own token accounts, be a mint authority, hold SOL -- and only its
program can sign for it, with `invoke_signed` and the seeds. The escrow's
vault, the staking pool's reward mint and the multisig's vault all work this
way. What the program *chooses* to sign is the entire security model.

### The accumulator

```
reward_per_token += elapsed * rate / total_staked      // on every action
earned = amount * (reward_per_token - reward_per_token_paid)
```

O(1) per action no matter how many stakers, and exact up to rounding.

## Common Pitfalls

1. **Comparing a key without checking it signed** -- the key is just a value in the account list
2. **Reading an account without checking its owner** -- reading foreign accounts is allowed, trusting them isn't
3. **One owner check for several account types** -- add a type tag (Anchor's discriminator)
4. **CPI into an unchecked program id** -- especially with `invoke_signed`: you hand it your PDA's signature
5. **Two parameters, one account** -- `from == to`, `source == destination`: decide what it means or reject it
6. **Writing stale state after a CPI** -- the callee may have changed the account
7. **Rounding up payouts** -- dust adds up, and an attacker can loop

## Running This Module

From `06-blockchain-solana/`:

```bash
cargo run  -p m28-smart-contracts -- 1                    # your code (1-4, bonus, all)
cargo test -p m28-smart-contracts-tests --features mine   # test your code
cargo run  -p m28-smart-contracts-solution -- all         # the reference solution
cargo test -p m28-smart-contracts-tests                   # 28 tests against the solution
```

Optional: `./build-sbf.sh solution 28` and `--features sbf` run the
programs as SBF bytecode in LiteSVM (4 tests) -- see
[GETTING_STARTED.md](GETTING_STARTED.md).

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (a multi-signature wallet, an escrow, a
staking mechanism, a security review).

- **The programs are native** (`solana-program`), not Anchor: for a security
  module, writing every check by hand shows what Anchor's constraints do.
  ANSWERS.md maps each bug to the Anchor feature that prevents it.
- **The security review uses six of the
  [sealevel-attacks](https://github.com/coral-xyz/sealevel-attacks)
  categories.** Not covered: closing accounts without wiping them (module 26
  does it right), non-canonical bumps (module 26's answers), and
  `init_if_needed` reinitialization.
- **Program upgrades and compute budget** from the outline aren't exercises:
  `solsim` has no upgradeable loader or compute metering. The `sbf` tests
  print compute units used on the real VM.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[29 · NFTs and Tokens](../29-nfts-tokens/)
