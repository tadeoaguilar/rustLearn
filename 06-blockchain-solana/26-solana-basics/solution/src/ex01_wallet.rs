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
    Keypair::new()
}

/// The same keypair every time for the same 32-byte seed. (Real wallets
/// derive the seed from a mnemonic phrase; that's BIP-39 + SLIP-10.)
pub fn wallet_from_seed(seed: &[u8; 32]) -> Keypair {
    Keypair::new_from_array(*seed)
}

/// `solana-keygen`'s file format: a JSON array of the 64 keypair bytes
/// (32 secret, then 32 public), e.g. `[12,201,...]`.
pub fn to_keypair_file(keypair: &Keypair) -> String {
    serde_json::to_string(&keypair.to_bytes().to_vec()).expect("bytes serialize")
}

/// Read a keypair file, checking that the public half belongs to the
/// secret half.
pub fn from_keypair_file(json: &str) -> Result<Keypair, WalletError> {
    let bytes: Vec<u8> = serde_json::from_str(json).map_err(|_| WalletError::BadKeypairFile)?;
    let bytes: [u8; 64] = bytes.try_into().map_err(|_| WalletError::BadKeypairFile)?;
    let secret: [u8; 32] = bytes[..32].try_into().expect("32 bytes");
    let keypair = Keypair::new_from_array(secret);
    if keypair.pubkey().as_ref() != &bytes[32..] {
        return Err(WalletError::MismatchedKeypair);
    }
    Ok(keypair)
}

/// Sign arbitrary bytes ("sign in with Solana" uses exactly this).
pub fn sign_message(keypair: &Keypair, message: &[u8]) -> Signature {
    keypair.sign_message(message)
}

/// Does `signature` over `message` come from `signer`'s secret key?
pub fn verify(signer: &Pubkey, message: &[u8], signature: &Signature) -> bool {
    signature.verify(signer.as_ref(), message)
}

/// Parse a SOL amount written in decimal ("1.5", "0.000000001", "42") into
/// lamports, exactly: no floating point, at most 9 decimals, no overflow.
pub fn parse_sol(text: &str) -> Result<u64, WalletError> {
    let bad = || WalletError::BadAmount(text.to_string());
    let (whole, frac) = match text.split_once('.') {
        Some((w, f)) => (w, f),
        None => (text, ""),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || (text.contains('.') && !digits(frac)) || frac.len() > 9 {
        return Err(bad());
    }
    let whole: u64 = whole.parse().map_err(|_| bad())?;
    let frac: u64 = if frac.is_empty() {
        0
    } else {
        format!("{frac:0<9}").parse().map_err(|_| bad())?
    };
    whole
        .checked_mul(LAMPORTS_PER_SOL)
        .and_then(|l| l.checked_add(frac))
        .ok_or_else(bad)
}

/// Lamports as SOL for people: `1_500_000_000` -> `"1.5 SOL"`, `1` ->
/// `"0.000000001 SOL"`, `2_000_000_000` -> `"2 SOL"`.
pub fn format_sol(lamports: u64) -> String {
    let whole = lamports / LAMPORTS_PER_SOL;
    let frac = lamports % LAMPORTS_PER_SOL;
    if frac == 0 {
        format!("{whole} SOL")
    } else {
        let frac = format!("{frac:09}");
        format!("{whole}.{} SOL", frac.trim_end_matches('0'))
    }
}
