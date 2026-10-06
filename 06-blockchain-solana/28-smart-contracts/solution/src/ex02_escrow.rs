//! Exercise 2: a token escrow -- a trustless swap.
//!
//! The maker offers `offer_amount` of token A for `want_amount` of token B.
//! *Make* moves the A tokens into a vault: a token account at the PDA
//! `["vault", escrow]` whose token-owner is the escrow PDA, so only this
//! program can move them. *Take* is one atomic instruction: the taker's B
//! tokens go to the maker, the vault's A tokens go to the taker, and the
//! vault and escrow accounts are closed, rent back to the maker. *Cancel*
//! returns the A tokens. Neither side can cheat: either both transfers
//! happen or neither does.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::util::{
    close_account, close_token_account, create_pda_account, create_token_account, require_pda,
    require_signer, token_account, token_transfer,
};

solana_program::declare_id!("CyFK1UjMseH6EkvPh4Tkmjyhe9VLX7qXfVbk73xfGj7d");

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum EscrowError {
    #[error("amounts must be greater than zero")]
    ZeroAmount = 0,
    #[error("a token account has the wrong mint")]
    WrongMint = 1,
    #[error("a token account belongs to the wrong owner")]
    WrongTokenOwner = 2,
    #[error("only the maker may do that")]
    NotTheMaker = 3,
    #[error("the account doesn't match the escrow")]
    EscrowMismatch = 4,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Escrow {
    pub maker: Pubkey,
    pub id: u64,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    /// Where the maker receives token B (checked at take).
    pub maker_receive_b: Pubkey,
    pub offer_amount: u64,
    pub want_amount: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Escrow {
    pub const SPACE: usize = 32 + 8 + 32 + 32 + 32 + 8 + 8 + 1 + 1;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum EscrowInstruction {
    /// `[maker (s, w), mint A, mint B, maker's A account (w), maker's B account,
    ///   escrow (w), vault (w), system program, token program]`
    Make {
        id: u64,
        offer_amount: u64,
        want_amount: u64,
    },
    /// `[taker (s, w), maker (w), escrow (w), vault (w), taker's A account (w),
    ///   taker's B account (w), maker's B account (w), token program]`
    Take,
    /// `[maker (s, w), escrow (w), vault (w), maker's A account (w), token program]`
    Cancel,
}

// ------------------------------------------------------------------ client

pub fn escrow_address(maker: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"escrow", maker.as_ref(), &id.to_le_bytes()], &ID).0
}

pub fn vault_address(escrow: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", escrow.as_ref()], &ID).0
}

fn build(accounts: Vec<AccountMeta>, data: EscrowInstruction) -> Instruction {
    Instruction {
        program_id: ID,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

#[allow(clippy::too_many_arguments)] // one per account, as on the wire
pub fn make(
    maker: &Pubkey,
    id: u64,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    maker_a: &Pubkey,
    maker_b: &Pubkey,
    offer_amount: u64,
    want_amount: u64,
) -> Instruction {
    let escrow = escrow_address(maker, id);
    build(
        vec![
            AccountMeta::new(*maker, true),
            AccountMeta::new_readonly(*mint_a, false),
            AccountMeta::new_readonly(*mint_b, false),
            AccountMeta::new(*maker_a, false),
            AccountMeta::new_readonly(*maker_b, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault_address(&escrow), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        EscrowInstruction::Make {
            id,
            offer_amount,
            want_amount,
        },
    )
}

pub fn take(
    taker: &Pubkey,
    maker: &Pubkey,
    escrow: &Pubkey,
    taker_a: &Pubkey,
    taker_b: &Pubkey,
    maker_b: &Pubkey,
) -> Instruction {
    build(
        vec![
            AccountMeta::new(*taker, true),
            AccountMeta::new(*maker, false),
            AccountMeta::new(*escrow, false),
            AccountMeta::new(vault_address(escrow), false),
            AccountMeta::new(*taker_a, false),
            AccountMeta::new(*taker_b, false),
            AccountMeta::new(*maker_b, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        EscrowInstruction::Take,
    )
}

pub fn cancel(maker: &Pubkey, escrow: &Pubkey, maker_a: &Pubkey) -> Instruction {
    build(
        vec![
            AccountMeta::new(*maker, true),
            AccountMeta::new(*escrow, false),
            AccountMeta::new(vault_address(escrow), false),
            AccountMeta::new(*maker_a, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        EscrowInstruction::Cancel,
    )
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match EscrowInstruction::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        EscrowInstruction::Make {
            id,
            offer_amount,
            want_amount,
        } => make_offer(program_id, accounts, id, offer_amount, want_amount),
        EscrowInstruction::Take => take_offer(program_id, accounts),
        EscrowInstruction::Cancel => cancel_offer(program_id, accounts),
    }
}

/// A token account of `mint` owned (token-wise) by `owner`.
fn check_token_account(info: &AccountInfo, mint: &Pubkey, owner: &Pubkey) -> ProgramResult {
    let account = token_account(info)?;
    if account.mint != *mint {
        return Err(EscrowError::WrongMint.into());
    }
    if account.owner != *owner {
        return Err(EscrowError::WrongTokenOwner.into());
    }
    Ok(())
}

fn make_offer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    offer_amount: u64,
    want_amount: u64,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let maker = next_account_info(iter)?;
    let mint_a = next_account_info(iter)?;
    let mint_b = next_account_info(iter)?;
    let maker_a = next_account_info(iter)?;
    let maker_b = next_account_info(iter)?;
    let escrow = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(maker)?;
    if offer_amount == 0 || want_amount == 0 {
        return Err(EscrowError::ZeroAmount.into());
    }
    check_token_account(maker_a, mint_a.key, maker.key)?;
    check_token_account(maker_b, mint_b.key, maker.key)?;

    let id_bytes = id.to_le_bytes();
    let bump = require_pda(
        escrow,
        &[b"escrow", maker.key.as_ref(), &id_bytes],
        program_id,
    )?;
    let vault_bump = require_pda(vault, &[b"vault", escrow.key.as_ref()], program_id)?;
    create_pda_account(
        maker,
        escrow,
        system_program,
        Escrow::SPACE,
        program_id,
        &[b"escrow", maker.key.as_ref(), &id_bytes, &[bump]],
    )?;
    // The vault's token-owner is the escrow PDA: only we can sign for it.
    create_token_account(
        maker,
        vault,
        mint_a,
        escrow.key,
        system_program,
        token_program,
        &[b"vault", escrow.key.as_ref(), &[vault_bump]],
    )?;
    token_transfer(token_program, maker_a, vault, maker, offer_amount, &[])?;

    let state = Escrow {
        maker: *maker.key,
        id,
        mint_a: *mint_a.key,
        mint_b: *mint_b.key,
        maker_receive_b: *maker_b.key,
        offer_amount,
        want_amount,
        bump,
        vault_bump,
    };
    let bytes = borsh::to_vec(&state).map_err(|_| ProgramError::InvalidAccountData)?;
    escrow.try_borrow_mut_data()?[..bytes.len()].copy_from_slice(&bytes);
    Ok(())
}

fn load_escrow(
    program_id: &Pubkey,
    escrow: &AccountInfo,
    vault: &AccountInfo,
) -> Result<Escrow, ProgramError> {
    if escrow.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let state = Escrow::deserialize(&mut &escrow.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let expected_vault = Pubkey::create_program_address(
        &[b"vault", escrow.key.as_ref(), &[state.vault_bump]],
        program_id,
    )?;
    if *vault.key != expected_vault {
        return Err(EscrowError::EscrowMismatch.into());
    }
    Ok(state)
}

fn take_offer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let taker = next_account_info(iter)?;
    let maker = next_account_info(iter)?;
    let escrow = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let taker_a = next_account_info(iter)?;
    let taker_b = next_account_info(iter)?;
    let maker_b = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(taker)?;
    let state = load_escrow(program_id, escrow, vault)?;
    if *maker.key != state.maker || *maker_b.key != state.maker_receive_b {
        return Err(EscrowError::EscrowMismatch.into());
    }
    check_token_account(taker_a, &state.mint_a, taker.key)?;
    check_token_account(taker_b, &state.mint_b, taker.key)?;

    // 1. taker pays the maker (the taker signed)
    token_transfer(
        token_program,
        taker_b,
        maker_b,
        taker,
        state.want_amount,
        &[],
    )?;
    // 2. the vault pays the taker, then closes (the escrow PDA signs)
    let escrow_seeds: &[&[u8]] = &[
        b"escrow",
        state.maker.as_ref(),
        &state.id.to_le_bytes(),
        &[state.bump],
    ];
    token_transfer(
        token_program,
        vault,
        taker_a,
        escrow,
        state.offer_amount,
        &[escrow_seeds],
    )?;
    close_token_account(token_program, vault, maker, escrow, &[escrow_seeds])?;
    // 3. the escrow account's rent goes back to the maker
    close_account(escrow, maker)
}

fn cancel_offer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let maker = next_account_info(iter)?;
    let escrow = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let maker_a = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(maker)?;
    let state = load_escrow(program_id, escrow, vault)?;
    if *maker.key != state.maker {
        return Err(EscrowError::NotTheMaker.into());
    }
    check_token_account(maker_a, &state.mint_a, maker.key)?;
    let escrow_seeds: &[&[u8]] = &[
        b"escrow",
        state.maker.as_ref(),
        &state.id.to_le_bytes(),
        &[state.bump],
    ];
    token_transfer(
        token_program,
        vault,
        maker_a,
        escrow,
        state.offer_amount,
        &[escrow_seeds],
    )?;
    close_token_account(token_program, vault, maker, escrow, &[escrow_seeds])?;
    close_account(escrow, maker)
}
