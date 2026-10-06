# Exercises: Smart Contracts

Programs that hold other people's money are written defensively: every
account is checked, every number is checked, state is changed before control
leaves the program, and nothing the caller passes in is trusted until proven.
This module writes four such programs natively (on `solana-program`, as in
module 26), and reviews a fifth.

| Exercise | Program | Teaches |
|---|---|---|
| 1 | multisig | executing arbitrary stored instructions with a PDA signature |
| 2 | escrow | token CPIs, PDA-owned token accounts, atomic swaps |
| 3 | staking | time-based rewards with an accumulator; rounding |
| 4 | security review | six classic vulnerabilities, exploited and fixed |
| bonus | multisig governance | a program changing its own rules through itself |

**Setup**: `util.rs` (provided) has the helpers module 26 taught --
creating/closing PDA accounts, token `transfer`/`mint_to`/`close_account`
CPIs, loading token accounts with an owner check. Instruction enums, state
structs and client builders are written for you in each file (the tests use
them); you write the processors. Tokens use the real SPL Token program, which
`solsim` provides.

Errors: each program's error enum becomes `ProgramError::Custom(n)`.

---

## Exercise 1: A Multisig Wallet

**Difficulty**: Hard
**Time**: 2 hours

**Learning Objectives**:
- Store an instruction and execute it later with `invoke_signed`
- Model approvals robustly
- Order state changes and CPIs safely

A multisig (PDA `["multisig", creator, id]`) has 1-10 distinct owners and a
threshold. Its vault, PDA `["vault", multisig]`, is a System account holding
SOL (and may own token accounts).

1. `StoredInstruction` <-> `Instruction` conversions.
2. **Create**: `InvalidOwners` unless 1-10 distinct owners; `InvalidThreshold`
   unless 1 <= threshold <= owners. Allocate `Multisig::SPACE`.
3. **Propose** (owners only, `NotAnOwner`): create the proposal at
   `["proposal", multisig, proposal_count]`, sized with `Proposal::space`,
   approved by the proposer; increment `proposal_count`.
4. **Approve**: an owner, once (`AlreadyApproved`), not after execution
   (`AlreadyExecuted`). The proposal must belong to this multisig (`WrongMultisig`).
5. **Execute** (anyone): enough approvals *from current owners*
   (`NotEnoughApprovals`), not already executed, the right vault. Mark it
   executed, then CPI the stored instruction signed by the vault. The
   instruction's accounts (and program) come after the four fixed accounts.

**Questions**:
1. Why set `executed = true` *before* the CPI?
2. Why does `approval_count` ignore approvals from accounts that are no
   longer owners?

---

## Exercise 2: A Token Escrow

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Hold tokens in a token account owned by a PDA
- Validate token accounts (mint and owner) before moving anything
- Close accounts and return rent

`make(id, offer_amount, want_amount)`: the maker locks `offer_amount` of
mint A in a vault (a token account at PDA `["vault", escrow]`, token-owned by
the escrow PDA `["escrow", maker, id]`) and asks `want_amount` of mint B.

1. **Make**: `ZeroAmount` for zero amounts; the maker's A account must hold
   mint A and belong to the maker, likewise the B account where they'll be
   paid (`WrongMint`, `WrongTokenOwner`). Create escrow and vault, move the
   tokens in, record everything.
2. **Take**: the accounts must match the escrow (`EscrowMismatch`: the vault,
   the maker, the maker's B account); the taker's accounts must be theirs and
   of the right mints. Pay the maker, pay the taker from the vault (signed
   by the escrow PDA), close the vault and the escrow -- rent to the maker.
3. **Cancel**: maker only (`NotTheMaker`); tokens back, accounts closed.

**Question**: what would a taker gain if `take` didn't check that the
"maker's B account" is the one recorded at `make`?

---

## Exercise 3: Staking with Continuous Rewards

**Difficulty**: Hard
**Time**: 2 hours

**Learning Objectives**:
- Pay rewards per second to any number of stakers in O(1)
- Use scaled integer arithmetic and reason about rounding
- Let a PDA be a mint authority

A pool (PDA `["pool", stake_mint]`) emits `reward_rate` reward tokens per
second, shared by stake. `reward_per_token` (times `PRECISION`) is what one
staked unit has earned since the start.

1. `accrue(pool, now)`: add `elapsed * rate * PRECISION / total_staked`
   (nothing while nothing is staked); never move `last_update` backwards;
   checked (`MathOverflow`).
2. `settle(stake, rpt)`: `pending += amount * (rpt - paid) / PRECISION`; `paid = rpt`.
3. **InitPool**: the reward mint's mint authority must already be the pool
   (`InvalidRewardMint`). Create the pool and its stake vault
   (`["stake_vault", pool]`).
4. **Stake**: create the stake account (`["stake", pool, user]`) on first
   use; accrue, settle, move tokens in.
5. **Unstake**: accrue, settle, `InsufficientStake` if too much; tokens out,
   signed by the pool.
6. **Claim**: accrue, settle, mint `pending` to the user (the pool signs).

A stake account must belong to the signer and the pool (`NotTheStaker`).

**Question**: why is a newly created stake account's
`reward_per_token_paid` set to the pool's current value, not 0?

---

## Exercise 4: A Security Review

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Recognise the classic Solana vulnerabilities
- Understand each one by exploiting it
- Fix them without breaking honest use

`ex04_security/vulnerable.rs` is a SOL bank (deposit accounts, a vault PDA,
an admin sweep) with six bugs; the tests exploit every one of them:

| # | Bug | Where |
|---|-----|-------|
| 1 | missing signer check | `withdraw` |
| 2 | missing owner check | `admin_withdraw` |
| 3 | type confusion | `admin_withdraw` |
| 4 | arbitrary CPI | `withdraw` |
| 5 | integer underflow | `transfer` |
| 6 | duplicate mutable accounts | `transfer` |

`ex04_security/secure.rs` starts as a copy of it. Read the exploits in the
tests (`ex4_bug*`), find each bug, and fix it in `secure.rs`. The errors to
use are in `BankError`; the test only requires that each exploit fails and
honest use (`ex4_honest_use_of_the_secure_bank`) works.

**Question**: for each bug, which line of defence in Anchor (module 27)
would have prevented it?

---

## Bonus: Self-Governance

**Difficulty**: Medium
**Time**: 30 minutes

In `bonus_governance.rs`: `SetThreshold { threshold }` requires the *vault*
as a signer. Nobody holds the vault's key, so the only way to run it is
through a proposal, executed by the multisig program CPI-ing into itself.

1. `set_threshold(multisig, threshold)`: the instruction to propose.
2. The handler: the vault signed (`MissingRequiredSignature`), the new
   threshold is valid (`InvalidThreshold`).

**Question**: Solana forbids reentrancy -- program A calling B calling A.
Why is A calling A allowed, and why is it safe here?
