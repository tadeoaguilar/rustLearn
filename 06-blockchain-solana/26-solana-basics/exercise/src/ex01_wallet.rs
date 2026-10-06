//! Exercise 1: wallets.
//!
//! A Solana "wallet" is an Ed25519 keypair. The public key, written in
//! base58, *is* the address; the secret key signs transactions. Nothing about
//! the address exists on-chain until someone sends it lamports.

use solana_keypair::Keypair;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_signature::Signature;
use solana_signer::Signer;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WalletError {
    #[error("a keypair file is a JSON array of 64 bytes")]
    BadKeypairFile,
    #[error("the public half doesn't match the secret half")]
    MismatchedKeypair,
    #[error("not a SOL amount: {0:?}")]
    BadAmount(String),
}

/// A fresh random keypair (what `solana-keygen new` does).
pub fn new_wallet() -> Keypair {
    todo!("Exercise 1")
}

/// The same keypair every time for the same 32-byte seed. (Real wallets
/// derive the seed from a mnemonic phrase; that's BIP-39 + SLIP-10.)
pub fn wallet_from_seed(seed: &[u8; 32]) -> Keypair {
    todo!("Exercise 1")
}

/// `solana-keygen`'s file format: a JSON array of the 64 keypair bytes
/// (32 secret, then 32 public), e.g. `[12,201,...]`.
pub fn to_keypair_file(keypair: &Keypair) -> String {
    todo!("Exercise 1")
}

/// Read a keypair file, checking that the public half belongs to the
/// secret half.
pub fn from_keypair_file(json: &str) -> Result<Keypair, WalletError> {
    todo!("Exercise 1")
}

/// Sign arbitrary bytes ("sign in with Solana" uses exactly this).
pub fn sign_message(keypair: &Keypair, message: &[u8]) -> Signature {
    todo!("Exercise 1")
}

/// Does `signature` over `message` come from `signer`'s secret key?
pub fn verify(signer: &Pubkey, message: &[u8], signature: &Signature) -> bool {
    todo!("Exercise 1")
}

/// Parse a SOL amount written in decimal ("1.5", "0.000000001", "42") into
/// lamports, exactly: no floating point, at most 9 decimals, no overflow.
pub fn parse_sol(text: &str) -> Result<u64, WalletError> {
    todo!("Exercise 1")
}

/// Lamports as SOL for people: `1_500_000_000` -> `"1.5 SOL"`, `1` ->
/// `"0.000000001 SOL"`, `2_000_000_000` -> `"2 SOL"`.
pub fn format_sol(lamports: u64) -> String {
    todo!("Exercise 1")
}
