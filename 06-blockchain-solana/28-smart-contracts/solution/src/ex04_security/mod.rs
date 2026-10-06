//! Exercise 4: a security review.
//!
//! `vulnerable.rs` is a small SOL bank -- open a deposit account, deposit,
//! withdraw, transfer between deposits, and an admin sweep -- with six
//! classic Solana bugs in it. Each one lets an attacker take the vault. The
//! tests run an exploit against it for each bug (and they all work).
//! `secure.rs` is your copy to fix: same instructions, same accounts, but
//! every exploit must fail while honest use keeps working.
//!
//! | # | Bug | In |
//! |---|-----|----|
//! | 1 | missing signer check | `withdraw` |
//! | 2 | missing owner check (a fake bank account) | `admin_withdraw` |
//! | 3 | type confusion (a deposit passed as the bank) | `admin_withdraw` |
//! | 4 | arbitrary CPI (an unchecked "system program") | `withdraw` |
//! | 5 | integer underflow | `transfer` |
//! | 6 | duplicate mutable accounts | `transfer` |
//!
//! This file (state, instructions, builders) is shared by both programs and
//! is provided.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;

pub mod secure;
pub mod vulnerable;

pub mod vulnerable_id {
    solana_program::declare_id!("44DwNnJwF9VbyfwiHTMRu8HZus712p9rL79MXXJ2XJZ8");
}
pub mod secure_id {
    solana_program::declare_id!("7LiBiooeyHUpv4QhVguWaua4Q7dHKvasFzpG2tFb3ZZQ");
}

/// First byte of every account: what kind of account it is.
pub const BANK_TAG: u8 = 1;
pub const DEPOSIT_TAG: u8 = 2;

/// Both kinds of account are allocated this size (room to grow).
pub const ACCOUNT_SPACE: usize = 64;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Bank {
    pub tag: u8,
    pub admin: Pubkey,
    pub bump: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Deposit {
    pub tag: u8,
    pub owner: Pubkey,
    pub balance: u64,
    pub bump: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BankError {
    InsufficientBalance = 0,
    NotTheOwner = 1,
    NotTheAdmin = 2,
    WrongAccountType = 3,
    SameAccount = 4,
}

impl From<BankError> for solana_program::program_error::ProgramError {
    fn from(e: BankError) -> Self {
        solana_program::program_error::ProgramError::Custom(e as u32)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum BankInstruction {
    /// `[admin (s, w), bank (w), system program]`
    Init,
    /// `[owner (s, w), deposit (w), system program]`
    Open,
    /// `[owner (s, w), deposit (w), vault (w), system program]`
    Deposit { amount: u64 },
    /// `[owner (s), deposit (w), vault (w), destination (w), system program]`
    Withdraw { amount: u64 },
    /// `[owner (s), from deposit (w), to deposit (w)]`
    Transfer { amount: u64 },
    /// `[admin (s), bank, vault (w), destination (w), system program]`
    AdminWithdraw { amount: u64 },
}

pub fn bank_address(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"bank"], program_id).0
}

pub fn deposit_address(program_id: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"deposit", owner.as_ref()], program_id).0
}

pub fn vault_address(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault"], program_id).0
}

fn build(program_id: &Pubkey, accounts: Vec<AccountMeta>, data: BankInstruction) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

const SYSTEM: Pubkey = solana_system_interface::program::ID;

pub fn init(program_id: &Pubkey, admin: &Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*admin, true),
        AccountMeta::new(bank_address(program_id), false),
        AccountMeta::new_readonly(SYSTEM, false),
    ];
    build(program_id, accounts, BankInstruction::Init)
}

pub fn open(program_id: &Pubkey, owner: &Pubkey) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(deposit_address(program_id, owner), false),
        AccountMeta::new_readonly(SYSTEM, false),
    ];
    build(program_id, accounts, BankInstruction::Open)
}

pub fn deposit(program_id: &Pubkey, owner: &Pubkey, amount: u64) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(deposit_address(program_id, owner), false),
        AccountMeta::new(vault_address(program_id), false),
        AccountMeta::new_readonly(SYSTEM, false),
    ];
    build(program_id, accounts, BankInstruction::Deposit { amount })
}

pub fn withdraw(program_id: &Pubkey, owner: &Pubkey, amount: u64) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(deposit_address(program_id, owner), false),
        AccountMeta::new(vault_address(program_id), false),
        AccountMeta::new(*owner, false),
        AccountMeta::new_readonly(SYSTEM, false),
    ];
    build(program_id, accounts, BankInstruction::Withdraw { amount })
}

pub fn transfer(
    program_id: &Pubkey,
    owner: &Pubkey,
    to_owner: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(deposit_address(program_id, owner), false),
        AccountMeta::new(deposit_address(program_id, to_owner), false),
    ];
    build(program_id, accounts, BankInstruction::Transfer { amount })
}

pub fn admin_withdraw(program_id: &Pubkey, admin: &Pubkey, amount: u64) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(bank_address(program_id), false),
        AccountMeta::new(vault_address(program_id), false),
        AccountMeta::new(*admin, false),
        AccountMeta::new_readonly(SYSTEM, false),
    ];
    build(
        program_id,
        accounts,
        BankInstruction::AdminWithdraw { amount },
    )
}
