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
        todo!("Exercise 2")
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
    todo!("Exercise 2")
}

/// A token account of `mint` owned (token-wise) by `owner`.
fn check_token_account(info: &AccountInfo, mint: &Pubkey, owner: &Pubkey) -> ProgramResult {
    todo!("Exercise 2")
}

fn make_offer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    offer_amount: u64,
    want_amount: u64,
) -> ProgramResult {
    todo!("Exercise 2")
}

fn load_escrow(
    program_id: &Pubkey,
    escrow: &AccountInfo,
    vault: &AccountInfo,
) -> Result<Escrow, ProgramError> {
    todo!("Exercise 2")
}

fn take_offer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    todo!("Exercise 2")
}

fn cancel_offer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    todo!("Exercise 2")
}
