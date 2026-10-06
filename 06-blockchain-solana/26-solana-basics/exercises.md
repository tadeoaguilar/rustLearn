# Exercises: Solana Basics

Solana is a single global computer whose memory is made of **accounts**.
Programs are stateless code; every byte of state lives in an account that
some program owns. A **transaction** is a list of **instructions**, each
naming a program, the accounts it may touch, and some bytes of data. It runs
atomically: all instructions succeed or nothing changes.

| Concept | Here |
|---|---|
| keypairs, addresses, signatures | Exercise 1 |
| lamports, rent, transfers, atomic transactions | Exercise 2 |
| a program: entrypoint, logs, return data, owned accounts | Exercise 3 |
| instruction data and account state (Borsh) | Exercise 4 |
| Program Derived Addresses; creating and closing accounts | Exercise 5 |
| cross-program invocation (`invoke`, `invoke_signed`) | Exercise 6 |

**Setup**: no Solana toolchain is needed. Programs are ordinary Rust
functions using the real `solana-program` API, and the tests run them in
`solsim` (`../sim`), an in-process runtime that enforces Solana's account
rules. Deploying to a local validator or devnet, and running the same
programs as SBF bytecode in LiteSVM, are optional steps in
GETTING_STARTED.md.

All commands run from `06-blockchain-solana/`.

---

## Exercise 1: Wallets

**Difficulty**: Easy
**Time**: 30 minutes

**Learning Objectives**:
- Know that an address is just a public key
- Read and write `solana-keygen` keypair files
- Sign and verify messages
- Handle SOL amounts without floating point

In `ex01_wallet.rs`:

1. `new_wallet()` and `wallet_from_seed(&[u8; 32])` (deterministic).
2. `to_keypair_file` / `from_keypair_file`: the JSON array of 64 bytes that
   `solana-keygen new` writes (secret half, then public half). Reject files
   that aren't 64 bytes, and files whose public half doesn't match.
3. `sign_message` / `verify`.
4. `parse_sol("1.5") == Ok(1_500_000_000)`: exact decimal parsing, at most 9
   decimals, overflow is an error. `format_sol(1_500_000_000) == "1.5 SOL"`.

**Question**: a friend wants to send you SOL. What do you give them, and
what must never leave your machine? Does your address "exist" before they send?

---

## Exercise 2: Funding, Transfers and Rent

**Difficulty**: Easy
**Time**: 40 minutes

**Learning Objectives**:
- Build System program instructions
- See transactions succeed or fail as a whole
- Understand rent exemption

`Sim` plays the cluster: `airdrop` is the devnet faucet, `process` sends a
transaction.

1. `describe(&Account) -> AccountKind`: `Wallet`, `Program`, `Sysvar` or
   `Data { owner }`, judging by owner, data and the executable flag.
2. `transfer_ix(from, to, lamports)`.
3. `pay_many(sim, payer, payments)`: every payment in **one** transaction.
4. `spendable(sim, wallet)`: the balance minus the rent-exempt minimum.
5. `create_data_account(sim, payer, new, space, owner)`: a System
   `create_account` funded with exactly `sim.minimum_balance(space)`.

Try sending 1000 lamports to a fresh address. Why does it fail?

**Question**: why does Solana charge a deposit (rent exemption) per byte
of account data, rather than a fee per transaction only?

---

## Exercise 3: Hello, World

**Difficulty**: Easy
**Time**: 40 minutes

**Learning Objectives**:
- Write a program entrypoint
- Validate instruction data
- Check account ownership before writing

A program is one function:

```rust
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult
```

In `ex03_hello.rs`:

1. `hello(name, counter)`: the instruction -- data is the name's UTF-8 bytes;
   the optional counter is the only (writable) account.
2. `process`: the name must be 1-32 bytes of UTF-8 (else
   `InvalidInstructionData`). Log `Hello, <name>!` with `msg!`, and return it
   with `set_return_data`.
3. If a counter account is passed, it must be **owned by this program**
   (`IncorrectProgramId`) and writable (`InvalidAccountData`); increment the
   little-endian `u64` in its first 8 bytes (checked).
4. `read_counter(data)` for clients.

**Question**: the runtime already refuses writes to accounts a program
doesn't own. Why check the owner yourself?

---

## Exercise 4: Instruction Data and Account State

**Difficulty**: Medium
**Time**: 45 minutes

**Learning Objectives**:
- Encode instructions and state with Borsh
- Size accounts up front
- Derive PDAs on the client

In `ex04_notes_state.rs`, for the notes program of Exercise 5:

1. `NoteInstruction::{pack, unpack}` with Borsh; malformed data (including
   trailing bytes) is `InvalidInstructionData`.
2. `Note::{read, write}`: the account is `Note::SPACE` bytes, longer than an
   encoded note -- `read` decodes a prefix, `write` zero-fills the rest.
3. `note_address(author, id)`: the PDA with seeds `["note", author, id as
   8 little-endian bytes]`.
4. `create_note`, `update_note`, `delete_note`: instructions with the
   accounts listed on each `NoteInstruction` variant.

---

## Exercise 5: The Notes Program -- PDAs

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Create accounts at PDAs with `invoke_signed`
- Validate every account an instruction receives
- Close accounts and reclaim rent

In `ex05_notes.rs`:

1. `NoteError` with `impl From<NoteError> for ProgramError` (`Custom(n)`).
2. **Create**: author signed; title 1-32 bytes, body at most 280; the note
   account is the PDA for (author, id); the system program is the real one.
   Create the account with a CPI to the System program, signed with the PDA's
   seeds, funded to rent exemption; write the note (keep the bump).
3. **Update**: author signed; the note is owned by this program, names this
   author, and sits at the PDA for its (author, id, bump).
4. **Delete**: the same checks; move all its lamports to the author, shrink
   it to 0 bytes and assign it back to the System program.

**Questions**:
1. In `delete`, why reassign the account to the System program?
2. Why store the bump in the account?

---

## Exercise 6: Cross-Program Invocation -- a Vault

**Difficulty**: Medium
**Time**: 1 hour

**Learning Objectives**:
- Call another program with `invoke`
- Sign for a PDA with `invoke_signed`
- Encode instruction data by hand

Each user's vault is the PDA `["vault", user]`: a System account holding
lamports. In `ex06_vault.rs`:

1. `encode` / `decode`: `[tag, amount as u64 LE]`, exactly 9 bytes, tags
   `DEPOSIT = 0`, `WITHDRAW = 1`.
2. `vault_address`, `deposit`, `withdraw`: accounts `[user (signer,
   writable), vault (writable), system program]`.
3. `process`: the user signed; the vault is the user's PDA (`InvalidSeeds`);
   the system program is real. Deposit is a System `transfer` from the user
   via `invoke`; withdraw is a `transfer` from the vault via `invoke_signed`.

**Question**: in `deposit`, the vault program asks the System program to
move the *user's* lamports. What stops a malicious program from doing that
to anyone?

---

## Bonus: Reading Program Accounts

**Difficulty**: Easy
**Time**: 20 minutes

A frontend lists "my notes" with `getProgramAccounts` and a `memcmp`
filter. In `bonus_client.rs`, with `Sim::program_accounts`:

1. `memcmp_filter(sim, program_id, offset, bytes)`.
2. `notes_by_author` (the author is at offset 0), sorted by id.
3. `next_note_id`.
