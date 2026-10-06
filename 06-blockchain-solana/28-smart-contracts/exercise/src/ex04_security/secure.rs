//! YOUR FIX of the bank (Exercise 4). It starts as an exact copy of
//! `vulnerable.rs`. Review it, find the six bugs listed in `mod.rs`, and fix
//! them here -- same instructions, same accounts, so the tests can run the
//! same exploits against it. Honest use must keep working.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use super::{ACCOUNT_SPACE, BANK_TAG, Bank, BankError, BankInstruction, DEPOSIT_TAG, Deposit};
use crate::util::create_pda_account;

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match BankInstruction::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)? {
        BankInstruction::Init => init(program_id, accounts),
        BankInstruction::Open => open(program_id, accounts),
        BankInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
        BankInstruction::Withdraw { amount } => withdraw(program_id, accounts, amount),
        BankInstruction::Transfer { amount } => transfer(program_id, accounts, amount),
        BankInstruction::AdminWithdraw { amount } => admin_withdraw(program_id, accounts, amount),
    }
}

fn save<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    let bytes = borsh::to_vec(state).map_err(|_| ProgramError::InvalidAccountData)?;
    info.try_borrow_mut_data()?[..bytes.len()].copy_from_slice(&bytes);
    Ok(())
}

fn read<T: BorshDeserialize>(info: &AccountInfo) -> Result<T, ProgramError> {
    T::deserialize(&mut &info.try_borrow_data()?[..]).map_err(|_| ProgramError::InvalidAccountData)
}

/// A System transfer out of the vault, signed by the vault PDA.
fn transfer_ix(
    system_program: &Pubkey,
    vault: &Pubkey,
    destination: &Pubkey,
    amount: u64,
) -> Instruction {
    let mut ix = solana_system_interface::instruction::transfer(vault, destination, amount);
    ix.program_id = *system_program;
    ix.accounts = vec![
        AccountMeta::new(*vault, true),
        AccountMeta::new(*destination, false),
    ];
    ix
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let admin = next_account_info(iter)?;
    let bank = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (expected, bump) = Pubkey::find_program_address(&[b"bank"], program_id);
    if *bank.key != expected {
        return Err(ProgramError::InvalidSeeds);
    }
    create_pda_account(
        admin,
        bank,
        system_program,
        ACCOUNT_SPACE,
        program_id,
        &[b"bank", &[bump]],
    )?;
    save(
        &Bank {
            tag: BANK_TAG,
            admin: *admin.key,
            bump,
        },
        bank,
    )
}

fn open(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let deposit = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let (expected, bump) =
        Pubkey::find_program_address(&[b"deposit", owner.key.as_ref()], program_id);
    if *deposit.key != expected {
        return Err(ProgramError::InvalidSeeds);
    }
    create_pda_account(
        owner,
        deposit,
        system_program,
        ACCOUNT_SPACE,
        program_id,
        &[b"deposit", owner.key.as_ref(), &[bump]],
    )?;
    save(
        &Deposit {
            tag: DEPOSIT_TAG,
            owner: *owner.key,
            balance: 0,
            bump,
        },
        deposit,
    )
}

fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let deposit = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    if deposit.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut state: Deposit = read(deposit)?;
    if state.owner != *owner.key {
        return Err(BankError::NotTheOwner.into());
    }
    let (expected_vault, _) = Pubkey::find_program_address(&[b"vault"], program_id);
    if *vault.key != expected_vault || *system_program.key != solana_system_interface::program::ID {
        return Err(ProgramError::InvalidArgument);
    }
    // The owner signs the transfer itself (the System program checks it).
    invoke(
        &solana_system_interface::instruction::transfer(owner.key, vault.key, amount),
        &[owner.clone(), vault.clone(), system_program.clone()],
    )?;
    state.balance = state
        .balance
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    save(&state, deposit)
}

fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let deposit = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let destination = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if deposit.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut state: Deposit = read(deposit)?;
    if state.owner != *owner.key {
        return Err(BankError::NotTheOwner.into());
    }
    if state.balance < amount {
        return Err(BankError::InsufficientBalance.into());
    }
    let (expected_vault, vault_bump) = Pubkey::find_program_address(&[b"vault"], program_id);
    if *vault.key != expected_vault {
        return Err(ProgramError::InvalidSeeds);
    }
    state.balance -= amount;
    save(&state, deposit)?;

    invoke_signed(
        &transfer_ix(system_program.key, vault.key, destination.key, amount),
        &[vault.clone(), destination.clone(), system_program.clone()],
        &[&[b"vault", &[vault_bump]]],
    )
}

fn transfer(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let from = next_account_info(iter)?;
    let to = next_account_info(iter)?;
    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if from.owner != program_id || to.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut from_state: Deposit = read(from)?;
    let mut to_state: Deposit = read(to)?;
    if from_state.owner != *owner.key {
        return Err(BankError::NotTheOwner.into());
    }
    from_state.balance = from_state.balance.wrapping_sub(amount);
    to_state.balance = to_state.balance.wrapping_add(amount);

    save(&from_state, from)?;
    save(&to_state, to)
}

fn admin_withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let admin = next_account_info(iter)?;
    let bank = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let destination = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let state: Bank = read(bank)?;
    if state.admin != *admin.key {
        return Err(BankError::NotTheAdmin.into());
    }
    let (expected_vault, vault_bump) = Pubkey::find_program_address(&[b"vault"], program_id);
    if *vault.key != expected_vault || *system_program.key != solana_system_interface::program::ID {
        return Err(ProgramError::InvalidArgument);
    }
    invoke_signed(
        &transfer_ix(system_program.key, vault.key, destination.key, amount),
        &[vault.clone(), destination.clone(), system_program.clone()],
        &[&[b"vault", &[vault_bump]]],
    )
}
