# Answers · 27 Anchor Framework

## Exercise 1: What does the discriminator protect against?

**Type confusion** ("type cosplay"). A program usually owns several kinds of
account; all of them pass an owner check. Without a type tag, an attacker
can pass an account of one type where another is expected, as long as the
bytes line up: a `Registry` whose first 32 bytes (`admin`) are read as a
`Profile`'s `owner`, say. Anchor writes `sha256("account:<Name>")[..8]` at
the start of every account it creates and checks it on every load, so a
`Registry` can never be deserialized as a `Profile`
(`AccountDiscriminatorMismatch`, 3002). Module 26's notes program got away
without one only because it had a single account type.

## Exercise 2: Why does a second vote fail?

The receipt is created with `init` at `["vote", poll, voter]` -- one address
per (poll, voter). The second time, the System program's `create_account`
finds the address in use and fails (`Custom(0)`, `AccountAlreadyInUse`), and
the whole transaction is rolled back, tally included. The *existence* of an
account is the record; no list of voters to search.

The voter pays (`payer = voter`): about 0.0014 SOL of rent per receipt (73 bytes). A
`close_receipt` instruction (`close = voter`, allowed once the poll has
ended) would give it back. Making the creator pay instead would let anyone
drain the creator by voting from many wallets.

## Exercise 3: With an `UncheckedAccount`

All of them: that the account is owned by this program (or an attacker's
lookalike passes), that it has the `Profile` discriminator (or a `Registry`
passes), that it deserializes, that it's writable, and -- after the handler
-- writing it back. `Account<'info, Profile>` does the first three on load
and the last on exit (for `mut` accounts). `UncheckedAccount` is for
accounts you genuinely don't interpret (a recipient of lamports, say), and
Anchor requires a `/// CHECK:` comment explaining why it's safe.

What `Account<T>` *doesn't* check is *which* profile it is: any valid
profile passes. That's what `seeds` (the address) and `has_one` (the owner
relation) add, and why `verify_profile` checks the registry's admin with
`has_one = admin`.
