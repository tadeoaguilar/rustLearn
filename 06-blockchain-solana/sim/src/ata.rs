//! The Associated Token Account program, enough of it for Phase 6: `Create`
//! and `CreateIdempotent`, at the real program id and with the real
//! instruction format. An associated token account (ATA) is the canonical
//! token account of a wallet for a mint: a PDA of this program with seeds
//! `[wallet, token program, mint]`, so anyone can compute where it is.

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_sdk_ids::system_program;

solana_program::declare_id!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

/// Where `wallet`'s token account for `mint` is.
pub fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    find(wallet, mint).0
}

fn find(wallet: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            wallet.as_ref(),
            spl_token_interface::ID.as_ref(),
            mint.as_ref(),
        ],
        &ID,
    )
}

/// Create `wallet`'s ATA for `mint`, paid by `payer`. Fails if it exists.
pub fn create_associated_token_account(
    payer: &Pubkey,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    build(payer, wallet, mint, 0)
}

/// Create the ATA unless it already exists (then do nothing).
pub fn create_associated_token_account_idempotent(
    payer: &Pubkey,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    build(payer, wallet, mint, 1)
}

fn build(payer: &Pubkey, wallet: &Pubkey, mint: &Pubkey, tag: u8) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(get_associated_token_address(wallet, mint), false),
            AccountMeta::new_readonly(*wallet, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        data: vec![tag],
    }
}

pub(crate) fn process<'info>(
    program_id: &'info Pubkey,
    accounts: &'info [AccountInfo<'info>],
    data: &'info [u8],
) -> ProgramResult {
    let idempotent = match data {
        [] | [0] => false,
        [1] => true,
        _ => return Err(ProgramError::InvalidInstructionData),
    };
    let [payer, ata, wallet, mint, system, token, ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if *token.key != spl_token_interface::ID || *system.key != system_program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let (expected, bump) = find(wallet.key, mint.key);
    if *ata.key != expected || program_id != &ID {
        return Err(ProgramError::InvalidSeeds);
    }
    if *ata.owner == spl_token_interface::ID && idempotent {
        let existing = spl_token_interface::state::Account::unpack(&ata.try_borrow_data()?)?;
        if existing.owner == *wallet.key && existing.mint == *mint.key {
            return Ok(());
        }
        return Err(ProgramError::IllegalOwner);
    }

    let space = spl_token_interface::state::Account::LEN;
    let lamports = Rent::get()?.minimum_balance(space);
    let seeds: &[&[u8]] = &[
        wallet.key.as_ref(),
        spl_token_interface::ID.as_ref(),
        mint.key.as_ref(),
        &[bump],
    ];
    invoke_signed(
        &solana_system_interface::instruction::create_account(
            payer.key,
            ata.key,
            lamports,
            space as u64,
            &spl_token_interface::ID,
        ),
        &[payer.clone(), ata.clone(), system.clone()],
        &[seeds],
    )?;
    invoke_signed(
        &spl_token_interface::instruction::initialize_account3(
            &spl_token_interface::ID,
            ata.key,
            mint.key,
            wallet.key,
        )?,
        &[ata.clone(), mint.clone(), token.clone()],
        &[],
    )
}
