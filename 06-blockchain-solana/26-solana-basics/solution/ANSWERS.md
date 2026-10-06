# Answers · 26 Solana Basics

## Exercise 1: What do you share?

Give them your **address** -- the base58 public key. The secret key (the
first 32 bytes of the keypair file, or the mnemonic it came from) never
leaves your machine: whoever has it can sign anything as you.

Your address doesn't exist on-chain until someone sends lamports to it:
an account with 0 lamports simply isn't stored. The first transfer to it
creates a System-owned account. That's also why a fresh keypair costs nothing.

## Exercise 2: Why rent?

Every validator keeps every account in memory/fast storage forever. A
transaction fee is paid once, but storage is a cost that never stops. Solana
makes the owner of the bytes post a deposit proportional to their size
(6,960 lamports per byte, counting 128 bytes of overhead per account -- about
0.007 SOL per KB): not spent, just locked, and returned in full when
the account is closed. It prices state, and gives everyone a reason to clean
up accounts they no longer need (Exercise 5's `delete`).

Sending 1000 lamports to a new address fails because the transaction would
leave a new account with lamports but below the minimum for its size (890,880
lamports for 0 bytes). Accounts are either rent-exempt or empty.

## Exercise 3: Why check the owner yourself?

The runtime stops you *writing* to an account you don't own -- but it doesn't
stop you *trusting* one. If the program read data from an account someone
else created (with the same layout, saying whatever the attacker likes), the
runtime wouldn't object: reading is allowed. Ownership is how you know your
program wrote the bytes, so check it before believing them. (Here the write
would also fail, but with a runtime error after the work was done instead of
a clear `IncorrectProgramId` up front.)

## Exercise 5

**Why reassign to the System program?** An account with 0 lamports is
deleted at the end of the transaction -- but within the transaction it still
exists. If it's left owned by our program with its old data, a later
instruction in the same transaction could refund it ("revive" it) and the
stale note would be back. Zeroing the data and handing it to the System
program means nothing of the note survives, and the address can be created
again later with a clean `create_account`.

**Why store the bump?** `find_program_address` tries bumps from 255
downwards until the address is off the Ed25519 curve; each attempt costs
compute (~1,500 CU). With the bump stored, later instructions call
`create_program_address` once. Always store and use the *canonical* bump
(the one `find_program_address` returns): accepting any bump the caller
supplies would let one (author, id) map to several valid addresses.

## Exercise 6: Who stops a program moving your lamports?

The runtime. A CPI can only pass on privileges the program itself received:
the user's account is a signer in the CPI only because the user signed the
*transaction*, for this instruction to this program. A malicious program
can't make a transfer from someone who didn't sign the transaction --
`invoke` would fail with a privilege escalation. The flip side: signing a
transaction hands your signature to every program it calls, which is why
wallets show you what a transaction does before you sign.
