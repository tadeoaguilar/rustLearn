# Answers · 29 NFTs and Tokens

## Exercise 1: Why pass the decimals?

As a guard against mistakes. "Send 1.5" means 1,500,000 base units for a
6-decimal token and 1,500,000,000 for a 9-decimal one. A client that
assumed the wrong decimals would send a thousand times too much with plain
`transfer`; with `transfer_checked`, the program compares the client's
decimals with the mint's and fails instead. The `_checked` variants also
take the mint account, so the program verifies the token account really is
for that mint. Wallets and hardware wallets use them so the amount on screen
is the amount moved.

## Exercise 2: Fake collection members

The `verified` flag. Naming a collection costs nothing; setting
`collection.verified = true` requires the signature of the collection's
update authority, which only the collection's owner has. A marketplace (or
Exercise 4's farm) must check `verified` and the collection's *mint
address* -- never the name, the symbol or the image, which anyone can copy.
The same goes for creators: royalties should go only to verified creators.

## Exercise 3: Why the machine is the mint authority

Because whoever holds the mint authority can mint more. If the buyer's key
were the authority, nothing would stop them minting 1,000 copies before (or
instead of) removing it, or skipping the metadata, the payment and the
limits -- each check would be a step the buyer could leave out of their own
transaction. With the machine PDA as authority, the only way to mint is
through the machine's code, which does every step and finally removes the
authority, leaving a supply of exactly 1.

## Bonus: Proof size

A proof is one sibling hash per level: 20 x 32 bytes = 640 bytes for a
million leaves (log2 of the count). The program stores only the 32-byte
root (real trees also keep a small buffer of recent roots and a "canopy" of
upper levels, so several transfers per block can use proofs against slightly
older roots, and proofs can be shorter). The leaves -- who owns what -- live
off-chain with indexers, and are recoverable from the transaction history.
Minting a million NFTs then costs one account's rent instead of a million.
