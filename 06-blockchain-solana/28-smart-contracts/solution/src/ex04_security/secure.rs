//! The bank, reviewed and fixed. Same instructions and accounts as
//! `vulnerable.rs`; each `FIX n` closes the matching `BUG n`.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
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

fn require_signer(info: &AccountInfo) -> ProgramResult {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// FIX 2 + 3: an account is only believed if this program owns it AND its
/// tag says it's the expected kind.
fn load_deposit(program_id: &Pubkey, info: &AccountInfo) -> Result<Deposit, ProgramError> {
    if info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let state = Deposit::deserialize(&mut &info.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if state.tag != DEPOSIT_TAG {
        return Err(BankError::WrongAccountType.into());
    }
    Ok(state)
}

fn load_bank(program_id: &Pubkey, info: &AccountInfo) -> Result<Bank, ProgramError> {
    if info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let state = Bank::deserialize(&mut &info.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if state.tag != BANK_TAG {
        return Err(BankError::WrongAccountType.into());
    }
    // ...and, belt and braces, it's THE bank: the PDA with the stored bump.
    let expected = Pubkey::create_program_address(&[b"bank", &[state.bump]], program_id)?;
    if *info.key != expected {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(state)
}

/// FIX 4: the vault is the PDA, and the program we CPI into is the real
/// System program.
fn check_vault_and_system(
    program_id: &Pubkey,
    vault: &AccountInfo,
    system_program: &AccountInfo,
) -> Result<u8, ProgramError> {
    if *system_program.key != solana_system_interface::program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let (expected, bump) = Pubkey::find_program_address(&[b"vault"], program_id);
    if *vault.key != expected {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump)
}

fn pay_from_vault<'a>(
    vault: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    amount: u64,
    bump: u8,
) -> ProgramResult {
    invoke_signed(
        &solana_system_interface::instruction::transfer(vault.key, destination.key, amount),
        &[vault.clone(), destination.clone(), system_program.clone()],
        &[&[b"vault", &[bump]]],
    )
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let admin = next_account_info(iter)?;
    let bank = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    require_signer(admin)?;
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
    require_signer(owner)?;
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
    require_signer(owner)?;
    let mut state = load_deposit(program_id, deposit)?;
    if state.owner != *owner.key {
        return Err(BankError::NotTheOwner.into());
    }
    check_vault_and_system(program_id, vault, system_program)?;
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
    require_signer(owner)?; // FIX 1
    let mut state = load_deposit(program_id, deposit)?;
    if state.owner != *owner.key {
        return Err(BankError::NotTheOwner.into());
    }
    let bump = check_vault_and_system(program_id, vault, system_program)?;
    state.balance = state
        .balance
        .checked_sub(amount)
        .ok_or(BankError::InsufficientBalance)?;
    save(&state, deposit)?;
    pay_from_vault(vault, destination, system_program, amount, bump)
}

fn transfer(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let from = next_account_info(iter)?;
    let to = next_account_info(iter)?;
    require_signer(owner)?;
    // FIX 6: two "different" accounts must be different.
    if from.key == to.key {
        return Err(BankError::SameAccount.into());
    }
    let mut from_state = load_deposit(program_id, from)?;
    let mut to_state = load_deposit(program_id, to)?;
    if from_state.owner != *owner.key {
        return Err(BankError::NotTheOwner.into());
    }
    // FIX 5: checked arithmetic; an insufficient balance is an error.
    from_state.balance = from_state
        .balance
        .checked_sub(amount)
        .ok_or(BankError::InsufficientBalance)?;
    to_state.balance = to_state
        .balance
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
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
    require_signer(admin)?;
    let state = load_bank(program_id, bank)?; // FIX 2, FIX 3
    if state.admin != *admin.key {
        return Err(BankError::NotTheAdmin.into());
    }
    let bump = check_vault_and_system(program_id, vault, system_program)?;
    pay_from_vault(vault, destination, system_program, amount, bump)
}
