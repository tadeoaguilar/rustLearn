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
        ProgramError::Custom(e as u32)
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
    match VendingInstruction::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        VendingInstruction::Initialize { id, config } => {
            init_machine(program_id, accounts, id, config)
        }
        VendingInstruction::Mint => mint_one(program_id, accounts),
    }
}

fn save<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    let bytes = borsh::to_vec(state).map_err(|_| ProgramError::InvalidAccountData)?;
    info.try_borrow_mut_data()?
        .get_mut(..bytes.len())
        .ok_or(ProgramError::AccountDataTooSmall)?
        .copy_from_slice(&bytes);
    Ok(())
}

fn init_machine(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    config: Config,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let authority = next_account_info(iter)?;
    let machine = next_account_info(iter)?;
    let treasury = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    require_signer(authority)?;
    if config.name_prefix.len() > Machine::MAX_PREFIX_LEN
        || config.base_uri.len() > Machine::MAX_BASE_URI_LEN
    {
        return Err(VendingError::BadConfig.into());
    }
    let id_bytes = id.to_le_bytes();
    let bump = require_pda(
        machine,
        &[b"machine", authority.key.as_ref(), &id_bytes],
        program_id,
    )?;
    create_pda_account(
        authority,
        machine,
        system_program,
        Machine::SPACE,
        program_id,
        &[b"machine", authority.key.as_ref(), &id_bytes, &[bump]],
    )?;
    let state = Machine {
        authority: *authority.key,
        id,
        treasury: *treasury.key,
        config,
        items_redeemed: 0,
        bump,
    };
    save(&state, machine)
}

fn mint_one(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let buyer = next_account_info(iter)?;
    let machine = next_account_info(iter)?;
    let treasury = next_account_info(iter)?;
    let new_mint = next_account_info(iter)?;
    let buyer_ata = next_account_info(iter)?;
    let metadata = next_account_info(iter)?;
    let record = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;
    let ata_program = next_account_info(iter)?;
    let metadata_program = next_account_info(iter)?;

    require_signer(buyer)?;
    check_token_program(token_program)?;
    if *ata_program.key != ATA_PROGRAM || *metadata_program.key != ex02_metadata::ID {
        return Err(VendingError::WrongProgram.into());
    }
    if machine.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut state = Machine::deserialize(&mut &machine.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if *treasury.key != state.treasury {
        return Err(VendingError::WrongTreasury.into());
    }
    if Clock::get()?.unix_timestamp < state.config.go_live {
        return Err(VendingError::NotLive.into());
    }
    if state.items_redeemed >= state.config.items_available {
        return Err(VendingError::SoldOut.into());
    }

    // Per-wallet limit: a record PDA per (machine, buyer), created on first purchase.
    let record_bump = require_pda(
        record,
        &[b"minted", machine.key.as_ref(), buyer.key.as_ref()],
        program_id,
    )?;
    if record.data_is_empty() {
        create_pda_account(
            buyer,
            record,
            system_program,
            1,
            program_id,
            &[
                b"minted",
                machine.key.as_ref(),
                buyer.key.as_ref(),
                &[record_bump],
            ],
        )?;
    } else if record.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut minted = MintRecord::deserialize(&mut &record.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if minted.count >= state.config.per_wallet_limit {
        return Err(VendingError::WalletLimit.into());
    }
    minted.count += 1;
    save(&minted, record)?;

    // Effects first: count the item before the CPIs.
    state.items_redeemed += 1;
    let n = state.items_redeemed;
    save(&state, machine)?;

    // 0. payment
    invoke(
        &solana_system_interface::instruction::transfer(
            buyer.key,
            treasury.key,
            state.config.price,
        ),
        &[buyer.clone(), treasury.clone(), system_program.clone()],
    )?;

    let machine_seeds: &[&[u8]] = &[
        b"machine",
        state.authority.as_ref(),
        &state.id.to_le_bytes(),
        &[state.bump],
    ];
    // 1-2. the mint, with the machine as mint authority
    let space = spl_token_interface::state::Mint::LEN;
    invoke(
        &solana_system_interface::instruction::create_account(
            buyer.key,
            new_mint.key,
            solana_program::rent::Rent::get()?.minimum_balance(space),
            space as u64,
            &spl_token_interface::ID,
        ),
        &[buyer.clone(), new_mint.clone(), system_program.clone()],
    )?;
    invoke(
        &spl_token_interface::instruction::initialize_mint2(
            &spl_token_interface::ID,
            new_mint.key,
            machine.key,
            None,
            0,
        )?,
        &[new_mint.clone(), token_program.clone()],
    )?;
    // 3. the buyer's token account, and the one token
    invoke(
        &Instruction {
            program_id: ATA_PROGRAM,
            accounts: vec![
                AccountMeta::new(*buyer.key, true),
                AccountMeta::new(*buyer_ata.key, false),
                AccountMeta::new_readonly(*buyer.key, false),
                AccountMeta::new_readonly(*new_mint.key, false),
                AccountMeta::new_readonly(solana_system_interface::program::ID, false),
                AccountMeta::new_readonly(spl_token_interface::ID, false),
            ],
            data: vec![0],
        },
        &[
            buyer.clone(),
            buyer_ata.clone(),
            new_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            ata_program.clone(),
        ],
    )?;
    invoke_signed(
        &spl_token_interface::instruction::mint_to(
            &spl_token_interface::ID,
            new_mint.key,
            buyer_ata.key,
            machine.key,
            &[],
            1,
        )?,
        &[
            new_mint.clone(),
            buyer_ata.clone(),
            machine.clone(),
            token_program.clone(),
        ],
        &[machine_seeds],
    )?;
    // 4. metadata (the machine signs as mint authority), then no more minting
    let args = CreateArgs {
        name: format!("{} #{n}", state.config.name_prefix),
        symbol: String::new(),
        uri: format!("{}/{n}.json", state.config.base_uri),
        seller_fee_basis_points: 0,
        creators: vec![],
        collection: None,
        is_mutable: false,
    };
    invoke_signed(
        &ex02_metadata::create(buyer.key, new_mint.key, machine.key, args),
        &[
            buyer.clone(),
            new_mint.clone(),
            machine.clone(),
            metadata.clone(),
            system_program.clone(),
            metadata_program.clone(),
        ],
        &[machine_seeds],
    )?;
    invoke_signed(
        &spl_token_interface::instruction::set_authority(
            &spl_token_interface::ID,
            new_mint.key,
            None,
            spl_token_interface::instruction::AuthorityType::MintTokens,
            machine.key,
            &[],
        )?,
        &[new_mint.clone(), machine.clone(), token_program.clone()],
        &[machine_seeds],
    )
}
