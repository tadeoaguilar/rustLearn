//! Exercise 6: cross-program invocation (CPI) -- a SOL vault.
//!
//! Every user gets a vault: a PDA `["vault", user]` of this program that
//! holds lamports. It's a plain System account (no data), so moving lamports
//! in or out is a System program `transfer` -- which this program calls:
//!
//! - **deposit**: `invoke` -- the user signed the transaction, and that
//!   signature carries over into the CPI
//! - **withdraw**: `invoke_signed` -- the *vault* must sign the transfer, and
//!   only this program can sign for its PDA, so only it can empty a vault --
//!   and it only does so for the vault's user
//!
//! The instruction data is laid out by hand here (`[tag, u64 LE]`) to show
//! what Borsh does for you.

use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction::transfer;

solana_program::declare_id!("G5zNmi2mvxTLT8NLKWyLykYypm9e54YNGqai5cyBEHt");

pub const DEPOSIT: u8 = 0;
pub const WITHDRAW: u8 = 1;

/// `user`'s vault address and bump.
pub fn vault_address(user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault", user.as_ref()], &ID)
}

/// `[tag, amount as 8 little-endian bytes]`
pub fn encode(tag: u8, amount: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(9);
    data.push(tag);
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

/// The inverse of `encode`; exactly 9 bytes or `InvalidInstructionData`.
pub fn decode(data: &[u8]) -> Result<(u8, u64), ProgramError> {
    match data {
        [tag @ (DEPOSIT | WITHDRAW), amount @ ..] if amount.len() == 8 => Ok((
            *tag,
            u64::from_le_bytes(amount.try_into().expect("8 bytes")),
        )),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn instruction(user: &Pubkey, tag: u8, amount: u64) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(vault_address(user).0, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        data: encode(tag, amount),
    }
}

/// Accounts: `[signer, writable] user`, `[writable] vault`, `[] system program`.
pub fn deposit(user: &Pubkey, amount: u64) -> Instruction {
    instruction(user, DEPOSIT, amount)
}

/// Same accounts as `deposit`.
pub fn withdraw(user: &Pubkey, amount: u64) -> Instruction {
    instruction(user, WITHDRAW, amount)
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let (tag, amount) = decode(data)?;
    let iter = &mut accounts.iter();
    let user = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if *system_program.key != solana_system_interface::program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let (expected, bump) = Pubkey::find_program_address(&[b"vault", user.key.as_ref()], program_id);
    if *vault.key != expected {
        return Err(ProgramError::InvalidSeeds);
    }
    let accounts = [user.clone(), vault.clone(), system_program.clone()];
    match tag {
        DEPOSIT => invoke(&transfer(user.key, vault.key, amount), &accounts),
        _ => invoke_signed(
            &transfer(vault.key, user.key, amount),
            &accounts,
            &[&[b"vault", user.key.as_ref(), &[bump]]],
        ),
    }
}
