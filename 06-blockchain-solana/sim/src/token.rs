//! Test helpers for SPL tokens: set up mints and token accounts in one call.
//!
//! These are infrastructure for the tests of modules 28-30. Module 29's
//! Exercise 1 asks you to build the same transactions yourself -- try it
//! before reading this file.

use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction::create_account;
use spl_token_interface::state::{Account as TokenAccount, Mint};

use crate::Sim;
use crate::ata::{create_associated_token_account, get_associated_token_address};

/// Create a mint with `decimals`, `authority` as mint authority, paid by
/// `payer`. Panics on failure (it's for test setup).
pub fn create_mint(sim: &mut Sim, payer: &Pubkey, authority: &Pubkey, decimals: u8) -> Pubkey {
    let mint = Pubkey::new_unique();
    let lamports = sim.minimum_balance(Mint::LEN);
    let ixs = [
        create_account(
            payer,
            &mint,
            lamports,
            Mint::LEN as u64,
            &spl_token_interface::ID,
        ),
        spl_token_interface::instruction::initialize_mint2(
            &spl_token_interface::ID,
            &mint,
            authority,
            None,
            decimals,
        )
        .expect("initialize_mint2"),
    ];
    sim.process(&ixs, &[*payer, mint]).expect("create mint");
    mint
}

/// Create `owner`'s associated token account for `mint`, paid by `payer`.
pub fn create_ata(sim: &mut Sim, payer: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    sim.process_ix(
        create_associated_token_account(payer, owner, mint),
        &[*payer],
    )
    .expect("create ATA");
    get_associated_token_address(owner, mint)
}

/// Create a token account for `mint` owned by `owner` (any key, PDAs
/// included) at a fresh address, paid by `payer`.
pub fn create_token_account(
    sim: &mut Sim,
    payer: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Pubkey {
    let account = Pubkey::new_unique();
    let lamports = sim.minimum_balance(TokenAccount::LEN);
    let ixs = [
        create_account(
            payer,
            &account,
            lamports,
            TokenAccount::LEN as u64,
            &spl_token_interface::ID,
        ),
        spl_token_interface::instruction::initialize_account3(
            &spl_token_interface::ID,
            &account,
            mint,
            owner,
        )
        .expect("initialize_account3"),
    ];
    sim.process(&ixs, &[*payer, account])
        .expect("create token account");
    account
}

/// Mint `amount` base units to `destination`; `authority` signs.
pub fn mint_to(
    sim: &mut Sim,
    mint: &Pubkey,
    authority: &Pubkey,
    destination: &Pubkey,
    amount: u64,
) {
    let ix = spl_token_interface::instruction::mint_to(
        &spl_token_interface::ID,
        mint,
        destination,
        authority,
        &[],
        amount,
    )
    .expect("mint_to");
    sim.process_ix(ix, &[*authority]).expect("mint");
}

/// A token account's balance in base units (0 if it doesn't exist).
pub fn balance(sim: &Sim, token_account: &Pubkey) -> u64 {
    token_account_state(sim, token_account).map_or(0, |a| a.amount)
}

/// The decoded token account, if it exists and is one.
pub fn token_account_state(sim: &Sim, token_account: &Pubkey) -> Option<TokenAccount> {
    let account = sim.account(token_account)?;
    (account.owner == spl_token_interface::ID)
        .then(|| TokenAccount::unpack(&account.data).ok())
        .flatten()
}

/// The decoded mint, if it exists and is one.
pub fn mint_state(sim: &Sim, mint: &Pubkey) -> Option<Mint> {
    let account = sim.account(mint)?;
    (account.owner == spl_token_interface::ID)
        .then(|| Mint::unpack(&account.data).ok())
        .flatten()
}
