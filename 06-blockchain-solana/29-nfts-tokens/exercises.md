# Exercises: NFTs and Tokens

Every token on Solana -- a stablecoin, a game currency, a one-of-one
artwork -- is an account of the same program, SPL Token. A **mint** defines
the token; **token accounts** hold balances; an owner's canonical token
account for a mint is its **associated token account** (ATA). An NFT is a
mint with 0 decimals and a supply of 1 whose mint authority has been
removed, plus **metadata** saying what it is.

**Setup**: the real SPL Token program and an ATA program are built into
`solsim`. Exercises 2-4 are programs; Exercise 1 is client code. Exercise 3
CPIs into Exercise 2's program, and Exercise 4 reads its accounts, so do
them in order. `util.rs` is module 28's (provided).

---

## Exercise 1: A Fungible Token

**Difficulty**: Easy
**Time**: 45 minutes

**Learning Objectives**:
- Create a mint and associated token accounts
- Mint, transfer, burn, freeze and thaw with the `_checked` instructions
- Fix a token's supply by removing its mint authority

In `ex01_fungible.rs`, using `spl_token_interface::instruction` and
`solsim::ata`:

1. `create_mint_ixs`: System `create_account` (owner: the token program,
   `Mint::LEN` bytes) then `initialize_mint2`.
2. `create_token(sim, payer, decimals, freezable)`.
3. `mint_to_wallet`: create the wallet's ATA *idempotently*, then
   `mint_to_checked`; `send`: the same for the recipient, then
   `transfer_checked`. One transaction each.
4. `burn`, `freeze`, `thaw`, `fix_supply` (`set_authority` to `None`),
   `balance_of`.
5. `format_amount(1_500_000, 6) == "1.5"` and `parse_amount`, exact.

**Question**: why do `transfer_checked` and `mint_to_checked` take the
decimals, when the program could read them from the mint?

---

## Exercise 2: NFTs and Metadata

**Difficulty**: Medium
**Time**: 2 hours

**Learning Objectives**:
- Attach data to a mint with a PDA owned by another program
- Decide what a signature proves: control of the mint, of a creator key, of a collection
- Mint a 1-of-1 NFT

In `ex02_metadata.rs` (the builders and `Metadata::read` are given):

1. **Create**: the payer and the **mint authority** signed, and the mint's
   authority really is that key (`NotMintAuthority`); name/symbol/URI at most
   32/10/200 bytes (`TooLong`); royalty at most 10000 bps (`BadRoyalty`); at
   most 5 creators whose shares add up to 100 (`BadCreators`). A creator is
   verified only if it's the signing mint authority; the collection starts
   unverified. Metadata lives at `["metadata", mint]`, `Metadata::SPACE` bytes.
2. **Update** (update authority only, `NotUpdateAuthority`; only while
   mutable, `Immutable`): name, URI, a new update authority, and
   `is_mutable = false` -- never back to `true`.
3. **VerifyCreator**: the signer must be a listed creator (`NotACreator`).
4. **VerifyCollection**: the signer must be the update authority of the
   *collection's* metadata (`NotUpdateAuthority`), and that collection must
   be the one the NFT names (`WrongCollection`).
5. `royalty_split(price, metadata)`.
6. `mint_nft`: one transaction -- the mint, the owner's ATA, mint 1, the
   metadata, remove the mint authority.

**Question**: anyone can create an NFT whose metadata names a famous
collection. What stops a marketplace from treating it as part of it?

---

## Exercise 3: A Vending Machine

**Difficulty**: Hard
**Time**: 2 hours

**Learning Objectives**:
- Make a PDA the mint authority of tokens a program creates
- Chain CPIs to four programs in one instruction
- Enforce price, supply, start time and per-wallet limits

`Initialize` stores a `Config` (prefix at most 24 bytes, base URI at most
150: `BadConfig`). `Mint` sells item `n = items_redeemed + 1`:

1. Check the programs passed (`WrongProgram`), the treasury (`WrongTreasury`),
   go-live (`NotLive`), supply (`SoldOut`), and the buyer's record at
   `["minted", machine, buyer]` -- create it on the first purchase -- against
   the limit (`WalletLimit`).
2. Count the item, then: pay the treasury; create and initialize the new
   mint (the buyer brings a fresh keypair) with the machine PDA as authority;
   create the buyer's ATA; mint 1; create metadata `"<prefix> #n"`,
   `"<base_uri>/n.json"` by CPI to Exercise 2's program, signed by the
   machine; remove the mint authority.

**Question**: why must the machine, not the buyer, be the mint authority
while the NFT is being made?

---

## Exercise 4: NFT Staking

**Difficulty**: Medium
**Time**: 1.5 hours

**Learning Objectives**:
- Gate a feature on a *verified* collection
- Custody an NFT in a PDA-owned token account
- Reward time staked

A farm (`["farm", collection]`) pays `rate` reward tokens per second per
staked NFT; the reward mint's authority must be the farm (`InvalidRewardMint`).

1. **Stake**: the metadata must be the real metadata of this mint
   (`WrongMetadata` if it's another NFT's) with the farm's collection,
   verified (`NotInCollection`). Move the NFT to a vault
   (`["nft_vault", mint]`, token-owned by the farm) and record owner and time
   at `["stake", mint]`.
2. **Unstake**: only the staker (`NotTheStaker`); NFT back, rewards minted,
   vault and record closed.

---

## Bonus: Compressed NFTs

**Difficulty**: Medium
**Time**: 45 minutes

In `bonus_compressed.rs`: leaf and node hashes (domain-separated), a
`MerkleTree` with `root` and `proof`, `verify`, and `replace_leaf` -- what a
compressed-NFT program does on-chain for a transfer, holding only the root.

**Question**: a tree of depth 20 holds a million NFTs. How big is a proof,
and what does the program store?
