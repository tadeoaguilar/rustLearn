# Exercises: Anchor Framework

Module 26 did by hand what every Solana program needs: parse instruction
data, check each account's owner, signer flag, address and type,
(de)serialize state, create accounts with CPIs. **Anchor** generates all of
that from declarations:

```rust
#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    pub authority: Signer<'info>,                          // must have signed
    #[account(mut, has_one = authority,                     // counter.authority == authority
              seeds = [b"counter", authority.key().as_ref()], bump = counter.bump)]
    pub counter: Account<'info, Counter>,                  // owned by us, discriminator matches
}
```

and the handler only holds the logic. All four exercises are one program,
`anchor_lab`; `lib.rs` (provided) routes each instruction to a handler
method on its accounts struct, which you write along with the constraints.

**Setup**: the Anchor *crate* (`anchor-lang` 1.2) only; the `anchor` CLI is
optional. Tests run the program natively in `solsim` (see module 26).
Error codes: Anchor's own are 2000-3999 (e.g. `ConstraintSeeds` = 2006,
`ConstraintHasOne` = 2001, `AccountDiscriminatorMismatch` = 3002), yours
start at 6000 in declaration order (`errors.rs`, provided).

---

## Exercise 1: Counter

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Create a PDA account with `init`
- Tie an account to a signer with `has_one`
- Return custom errors; close accounts

In `ex01_counter.rs`, add the constraints and the handlers:

| Instruction | Accounts | Logic |
|---|---|---|
| `initialize_counter` | `authority` pays; `counter` is created at `["counter", authority]`, `8 + Counter::INIT_SPACE` bytes | count 0, store authority and bump |
| `increment(by)` / `decrement(by)` / `reset` | `counter` writable, its authority signed, at its PDA (stored bump) | checked: `CounterOverflow`, `CounterUnderflow` |
| `close_counter` | same, plus `close = authority` | nothing -- the constraint does it |

**Question**: what does Anchor's 8-byte account discriminator protect
against? What would happen without it?

---

## Exercise 2: Voting

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Use instruction arguments in seeds (`#[instruction(...)]`)
- Size accounts with `#[max_len]` and `InitSpace`
- Read the clock; enforce "once per user" with an `init`

In `ex02_voting.rs`:

1. `create_poll(poll_id, question, options, ends_at)`: create the poll at
   `["poll", creator, poll_id as 8 LE bytes]`. Question 1-100 bytes
   (`BadQuestion`); 2-4 options of 1-32 bytes (`BadOptions`); `ends_at` after
   now (`EndsInThePast`). Votes start at 0 per option.
2. `vote(option)`: the poll at its PDA (stored seeds and bump); create a
   `VoteReceipt` at `["vote", poll, voter]`, paid by the voter. Voting closes
   at `ends_at` (`PollEnded`); the option must exist (`NoSuchOption`).
3. `close_poll`: only the creator, only once `now >= ends_at`
   (`PollRunning`); refund the rent to the creator.
4. `winners(&Poll)`: every option with the most votes.

**Question**: a second vote fails although `vote` never checks for one.
Why? Who pays for receipts, and how could they get that rent back?

---

## Exercise 3: Account Validation

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Combine constraints into an authorization model
- Attach custom errors to constraints (`@ LabError::X`)
- See which attacks `Account<T>` stops on its own

A global `Registry` (PDA `["registry"]`, an admin and a profile count) and a
`Profile` per wallet (PDA `["profile", owner]`). In `ex03_registry.rs`:

1. `valid_handle`: 3-16 characters of `a-z`, `0-9`, `_`.
2. `initialize_registry`: the signer becomes admin.
3. `create_profile(handle)`: `BadHandle` unless valid; the account is
   `8 + Profile::space(handle.len(), 0)` bytes; count it in the registry.
4. `verify_profile`: only the admin (`NotAdmin`), only once
   (`AlreadyVerified`) -- both as constraints.
5. `transfer_admin(new_admin)`: only the admin (`NotAdmin`).
6. `delete_profile`: only the owner; refund the rent; uncount it.

The tests also try type confusion: the registry passed as a profile, and a
perfect copy of a profile owned by another program.

**Question**: which of these checks would you have to write yourself if
`profile` were an `UncheckedAccount`?

---

## Exercise 4: A Client for Tests

**Difficulty**: Easy
**Time**: 40 minutes

**Learning Objectives**:
- Build instructions from Anchor's generated `accounts::` and `instruction::` structs
- Decode accounts safely
- Map error codes back to your error enum

In `ex04_client.rs` (client side only):

1. The PDA helpers: `counter_address`, `poll_address`, `receipt_address`,
   `registry_address`, `profile_address`.
2. Builders that derive every PDA themselves: `initialize_counter_ix`,
   `increment_ix`, `create_poll_ix`, `vote_ix`, `create_profile_ix`,
   `verify_profile_ix`.
3. `fetch::<T>(sim, key)`: `None` unless the account exists, is owned by
   this program, and has `T`'s discriminator.
4. `lab_error(&TxError)`: which `LabError` it was, if any.

Then write a few tests of your own with it (in `exercise/tests/`, a new
directory). The module's `tests/` crate is a model.

---

## Bonus: Growing Accounts with `realloc`

**Difficulty**: Medium
**Time**: 30 minutes

A profile starts just big enough for its handle. `set_bio(bio)` resizes it:
`realloc = 8 + Profile::space(handle_len, bio.len())`, paid by the owner,
`realloc::zero = false`. Bios are at most 200 bytes (`BioTooLong`). Check
that the account stays exactly rent-exempt as it grows and shrinks.
