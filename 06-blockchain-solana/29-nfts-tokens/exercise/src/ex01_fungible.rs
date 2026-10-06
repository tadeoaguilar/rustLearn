//! Exercise 1: a fungible token, from the client side.
//!
//! The SPL Token program is shared by every token on Solana: a *mint*
//! account defines a token (supply, decimals, who may mint and freeze), and
//! *token accounts* hold balances of one mint for one owner. An owner's
//! canonical token account for a mint is its *associated token account*
//! (ATA), a PDA anyone can compute. No code to deploy: a token is two
//! instructions to an existing program.

use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction::create_account;
use solsim::ata::{create_associated_token_account_idempotent, get_associated_token_address};
use solsim::{Sim, TxError};
use spl_token_interface::instruction as token_ix;
use spl_token_interface::instruction::AuthorityType;
use spl_token_interface::state::{Account as TokenAccount, Mint};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not an amount with at most {decimals} decimals: {text:?}")]
pub struct AmountError {
    pub text: String,
    pub decimals: u8,
}

/// The two instructions that make a mint: allocate it (owned by the token
/// program, rent-exempt), then initialize it.
pub fn create_mint_ixs(
    payer: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    lamports: u64,
) -> [Instruction; 2] {
    todo!("Exercise 1")
}

/// Create a new token: `payer` pays and becomes mint authority (and freeze
/// authority if `freezable`). Returns the mint's address.
pub fn create_token(
    sim: &mut Sim,
    payer: &Pubkey,
    decimals: u8,
    freezable: bool,
) -> Result<Pubkey, TxError> {
    todo!("Exercise 1")
}

/// Mint `amount` base units to `wallet`'s ATA, creating it if needed (the
/// authority pays). `mint_to_checked` also verifies the decimals. Returns the ATA.
pub fn mint_to_wallet(
    sim: &mut Sim,
    mint: &Pubkey,
    authority: &Pubkey,
    wallet: &Pubkey,
    amount: u64,
) -> Result<Pubkey, TxError> {
    todo!("Exercise 1")
}

/// Send `amount` from `from`'s ATA to `to`'s ATA, creating the recipient's
/// if needed (the sender pays for it).
pub fn send(
    sim: &mut Sim,
    mint: &Pubkey,
    from: &Pubkey,
    to: &Pubkey,
    amount: u64,
) -> Result<(), TxError> {
    todo!("Exercise 1")
}

/// Destroy `amount` of `owner`'s tokens (supply goes down).
pub fn burn(sim: &mut Sim, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Result<(), TxError> {
    todo!("Exercise 1")
}

/// Freeze `wallet`'s ATA: no transfers in or out until thawed.
pub fn freeze(
    sim: &mut Sim,
    mint: &Pubkey,
    freeze_authority: &Pubkey,
    wallet: &Pubkey,
) -> Result<(), TxError> {
    todo!("Exercise 1")
}

pub fn thaw(
    sim: &mut Sim,
    mint: &Pubkey,
    freeze_authority: &Pubkey,
    wallet: &Pubkey,
) -> Result<(), TxError> {
    todo!("Exercise 1")
}

/// Remove the mint authority for good: the supply can never grow again.
pub fn fix_supply(sim: &mut Sim, mint: &Pubkey, authority: &Pubkey) -> Result<(), TxError> {
    todo!("Exercise 1")
}

/// `wallet`'s balance of `mint` (0 without an ATA).
pub fn balance_of(sim: &Sim, mint: &Pubkey, wallet: &Pubkey) -> u64 {
    todo!("Exercise 1")
}

/// The mint's decimals (panics if `mint` isn't a mint: client-side helper).
pub fn decimals(sim: &Sim, mint: &Pubkey) -> u8 {
    Mint::unpack(sim.data(mint)).expect("a mint").decimals
}

/// Base units for people: `format_amount(1_500_000, 6) == "1.5"`.
pub fn format_amount(amount: u64, decimals: u8) -> String {
    todo!("Exercise 1")
}

/// The inverse: `parse_amount("1.5", 6) == Ok(1_500_000)`; exact, checked.
pub fn parse_amount(text: &str, decimals: u8) -> Result<u64, AmountError> {
    todo!("Exercise 1")
}
