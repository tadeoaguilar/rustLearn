//! Exercise 3: an NFT vending machine (a "candy machine").
//!
//! A creator sets up a machine: price, how many items, a go-live time, a
//! per-wallet limit, a name prefix and base URI. A buyer pays the price and
//! the machine mints a fresh NFT straight to them -- item `n` is named
//! `"<prefix> #n"` with URI `"<base_uri>/n.json"` -- all in one instruction,
//! by CPI into four programs:
//!
//! 1. System: create the new mint account (the buyer brings a fresh keypair)
//! 2. Token: initialize it with the *machine PDA* as mint authority
//! 3. ATA: create the buyer's token account; Token: mint 1 to it
//! 4. Metadata (Exercise 2): create its metadata, signed by the machine PDA
//!    as mint authority, then Token: remove the mint authority
//!
//! The machine is the only one who can mint, so the supply, price and
//! limits can't be bypassed.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

use crate::ex02_metadata::{self, CreateArgs};
use crate::util::{check_token_program, create_pda_account, require_pda, require_signer};

solana_program::declare_id!("8uSQPCFKnactpLSFTfEJVzQp7vmo3Lmo1UmkHmAVh36M");

/// The real Associated Token Account program (solsim provides it).
pub const ATA_PROGRAM: Pubkey =
    solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum VendingError {
    #[error("the machine isn't live yet")]
    NotLive = 0,
    #[error("sold out")]
    SoldOut = 1,
    #[error("this wallet reached its limit")]
    WalletLimit = 2,
    #[error("the treasury doesn't match the machine")]
    WrongTreasury = 3,
    #[error("a program account is not the expected program")]
    WrongProgram = 4,
    #[error("name prefix or base URI too long")]
    BadConfig = 5,
}

impl From<VendingError> for ProgramError {
    fn from(e: VendingError) -> Self {
        todo!("Exercise 3")
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub price: u64,
    pub items_available: u32,
    pub go_live: i64,
    pub per_wallet_limit: u8,
    pub name_prefix: String,
    pub base_uri: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Machine {
    pub authority: Pubkey,
    pub id: u64,
    pub treasury: Pubkey,
    pub config: Config,
    pub items_redeemed: u32,
    pub bump: u8,
}

impl Machine {
    pub const MAX_PREFIX_LEN: usize = 24;
    pub const MAX_BASE_URI_LEN: usize = 150;
    pub const SPACE: usize = 32
        + 8
        + 32
        + (8 + 4 + 8 + 1 + 4 + Self::MAX_PREFIX_LEN + 4 + Self::MAX_BASE_URI_LEN)
        + 4
        + 1;
}

/// How many NFTs a wallet bought from a machine.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct MintRecord {
    pub count: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum VendingInstruction {
    /// `[authority (s, w), machine (w), treasury, system program]`
    Initialize { id: u64, config: Config },
    /// `[buyer (s, w), machine (w), treasury (w), new mint (s, w), buyer's ATA (w),
    ///   metadata (w), mint record (w), system, token, ATA and metadata programs]`
    Mint,
}

// ------------------------------------------------------------------ client

pub fn machine_address(authority: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"machine", authority.as_ref(), &id.to_le_bytes()], &ID).0
}

pub fn mint_record_address(machine: &Pubkey, buyer: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"minted", machine.as_ref(), buyer.as_ref()], &ID).0
}

pub fn initialize(authority: &Pubkey, id: u64, treasury: &Pubkey, config: Config) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(machine_address(authority, id), false),
            AccountMeta::new_readonly(*treasury, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        data: borsh::to_vec(&VendingInstruction::Initialize { id, config }).expect("serialize"),
    }
}

/// Buy one NFT. `new_mint` is a fresh address that signs (a new keypair).
pub fn mint(buyer: &Pubkey, machine: &Pubkey, treasury: &Pubkey, new_mint: &Pubkey) -> Instruction {
    let ata = Pubkey::find_program_address(
        &[
            buyer.as_ref(),
            spl_token_interface::ID.as_ref(),
            new_mint.as_ref(),
        ],
        &ATA_PROGRAM,
    )
    .0;
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(*buyer, true),
            AccountMeta::new(*machine, false),
            AccountMeta::new(*treasury, false),
            AccountMeta::new(*new_mint, true),
            AccountMeta::new(ata, false),
            AccountMeta::new(ex02_metadata::metadata_address(new_mint), false),
            AccountMeta::new(mint_record_address(machine, buyer), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
            AccountMeta::new_readonly(ATA_PROGRAM, false),
            AccountMeta::new_readonly(ex02_metadata::ID, false),
        ],
        data: borsh::to_vec(&VendingInstruction::Mint).expect("serialize"),
    }
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    todo!("Exercise 3")
}

fn save<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    todo!("Exercise 3")
}

fn init_machine(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    config: Config,
) -> ProgramResult {
    todo!("Exercise 3")
}

fn mint_one(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    todo!("Exercise 3")
}
