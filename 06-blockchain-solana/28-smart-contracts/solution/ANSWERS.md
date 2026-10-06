# Answers · 28 Smart Contracts

## Exercise 1

**Why mark it executed before the CPI?** The stored instruction can be
anything, including a call back into code that could execute the same
proposal again (a CPI into another program that CPIs into the multisig would
be refused as reentrancy, but a direct self-CPI is allowed -- the bonus uses
exactly that). If `executed` were written *after* the CPI, any path that
re-entered `Execute` during the CPI would see `executed == false` and run the
instruction twice. "Checks, effects, interactions": validate, update your
own state, and only then hand control to someone else. (And don't write
state you loaded before the CPI afterwards -- the callee may have changed it.)

**Why ignore approvals from removed owners?** Owner sets change (through
proposals, like the bonus). If a removed owner's earlier approval still
counted, removing a compromised key wouldn't neutralise what it had already
approved. Storing approvers' keys -- rather than a bitmap by position --
also means removing an owner can't shift other owners' approvals onto the
wrong person.

## Exercise 2: Not checking the maker's B account

The taker could pass their *own* B account as "the maker's". Step 1 would
move the taker's B tokens to themselves (a no-op), step 2 would hand them the
vault's A tokens: the offer taken for free. Everything an instruction
receives that the program later *pays* must be bound to something the
counterparty committed to -- here, the account recorded at `make`.

## Exercise 3: Why start at the current `reward_per_token`?

`reward_per_token` is cumulative since the pool started. A stake account
created with `reward_per_token_paid = 0` would be credited
`amount * reward_per_token` at its first settle -- rewards for all the time
*before* it staked, taken from everyone else (or minted from nothing). Starting
at the current value means it earns only from now on.

On rounding: both divisions round down, so the pool never pays out more than
it emitted; the dust (a few units per settle) stays unminted. `PRECISION =
10^12` keeps the per-token rate from rounding to 0 when the rate is small and
the stake large.

## Exercise 4: Each bug, and Anchor's defence

| # | Bug | The fix | In Anchor |
|---|-----|---------|-----------|
| 1 | `withdraw` compares the owner's key but never checks it signed: anyone names the victim and withdraws to themselves | `require_signer(owner)` | `Signer<'info>` |
| 2 | `admin_withdraw` reads a "bank" without checking who owns the account: the attacker's own program creates one naming them admin | `bank.owner == program_id` | `Account<'info, Bank>` checks the owner |
| 3 | ... nor its type: a deposit account (owned by the bank, so it passes an owner check) has the owner where a bank has its admin | check the tag (and the PDA address) | the 8-byte discriminator; `seeds` |
| 4 | `withdraw` CPIs into whatever "system program" it's given, and the vault *signs* that CPI: the attacker's program receives a signing vault and empties it | check the program id | `Program<'info, System>` |
| 5 | `transfer` subtracts without checking (wrapping): a zero balance minus one is 18 quintillion | `checked_sub` -> `InsufficientBalance` | nothing automatic: use checked math (and keep `overflow-checks = true`) |
| 6 | `transfer` with `from == to`: both copies are read, then the second save overwrites the first with `balance + amount` | reject identical accounts | nothing automatic: Anchor deserializes each account separately too; add `constraint = from.key() != to.key()` |

Bugs 5 and 6 are why audits still find bugs in Anchor programs: frameworks
remove the account-validation boilerplate, not the logic errors.

## Bonus: Why is A -> A allowed?

Reentrancy is dangerous when a program is called back in the *middle* of an
operation, with its state half-updated, by code it doesn't control. Solana
refuses A -> B -> A for that reason. A -> A is a call the program makes to
itself, deliberately, at a point it chooses -- no foreign code sits in
between -- so it's no more dangerous than a function call. Here it's safe
because `Execute` marks the proposal executed *before* the CPI and doesn't
write the multisig afterwards, so the inner `SetThreshold` sees (and leaves)
consistent state.
