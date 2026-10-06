//! Exercise 4: NFT staking.
//!
//! Holders of a collection lock their NFT in the program and earn a reward
//! token for as long as it stays locked. The program must decide which NFTs
//! qualify -- by the metadata's *verified* collection, never by name or
//! symbol, which anyone can copy.
//!
//! - **Stake**: the NFT moves from the owner's ATA into a vault token account
//!   (PDA `["nft_vault", mint]`, token-owned by the farm PDA); a record
//!   `["stake", mint]` remembers who staked it and when.
//! - **Unstake**: only that owner; the NFT goes back, the reward mint (whose
//!   authority is the farm PDA) pays `seconds staked * rate`, and the vault
//!   and record are closed.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

use crate::ex02_metadata;
use crate::util::{
    close_account, close_token_account, create_pda_account, create_token_account, mint,
    require_pda, require_signer, token_account, token_mint_to, token_transfer,
};

solana_program::declare_id!("2GGHK4brT2see6dRkpaPo5UTb6AkvoSChYSSGZvHYAjk");

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum NftStakingError {
    #[error("not a verified member of the farm's collection")]
    NotInCollection = 0,
    #[error("the reward mint's authority must be the farm")]
    InvalidRewardMint = 1,
    #[error("only who staked it may unstake it")]
    NotTheStaker = 2,
    #[error("the metadata doesn't belong to this NFT")]
    WrongMetadata = 3,
}

impl From<NftStakingError> for ProgramError {
    fn from(e: NftStakingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Farm {
    /// The collection NFT's mint.
    pub collection: Pubkey,
    pub reward_mint: Pubkey,
    /// Reward base units per second per staked NFT.
    pub rate: u64,
    pub bump: u8,
}

impl Farm {
    pub const SPACE: usize = 32 + 32 + 8 + 1;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct StakeRecord {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub staked_at: i64,
    pub bump: u8,
}

impl StakeRecord {
    pub const SPACE: usize = 32 + 32 + 8 + 1;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum NftStakingInstruction {
    /// `[admin (s, w), farm (w), collection mint, reward mint, system program]`
    InitFarm { rate: u64 },
    /// `[owner (s, w), farm, nft mint, nft metadata, owner's NFT account (w),
    ///   vault (w), record (w), system program, token program]`
    Stake,
    /// `[owner (s, w), farm, nft mint, owner's NFT account (w), vault (w), record (w),
    ///   reward mint (w), owner's reward account (w), token program]`
    Unstake,
}

// ------------------------------------------------------------------ client

pub fn farm_address(collection: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"farm", collection.as_ref()], &ID).0
}

pub fn vault_address(nft_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"nft_vault", nft_mint.as_ref()], &ID).0
}

pub fn record_address(nft_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"stake", nft_mint.as_ref()], &ID).0
}

fn build(accounts: Vec<AccountMeta>, data: NftStakingInstruction) -> Instruction {
    Instruction {
        program_id: ID,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

pub fn init_farm(
    admin: &Pubkey,
    collection: &Pubkey,
    reward_mint: &Pubkey,
    rate: u64,
) -> Instruction {
    build(
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(farm_address(collection), false),
            AccountMeta::new_readonly(*collection, false),
            AccountMeta::new_readonly(*reward_mint, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        NftStakingInstruction::InitFarm { rate },
    )
}

pub fn stake(
    owner: &Pubkey,
    collection: &Pubkey,
    nft_mint: &Pubkey,
    owner_nft_account: &Pubkey,
) -> Instruction {
    build(
        vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(farm_address(collection), false),
            AccountMeta::new_readonly(*nft_mint, false),
            AccountMeta::new_readonly(ex02_metadata::metadata_address(nft_mint), false),
            AccountMeta::new(*owner_nft_account, false),
            AccountMeta::new(vault_address(nft_mint), false),
            AccountMeta::new(record_address(nft_mint), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        NftStakingInstruction::Stake,
    )
}

pub fn unstake(
    owner: &Pubkey,
    collection: &Pubkey,
    nft_mint: &Pubkey,
    owner_nft_account: &Pubkey,
    reward_mint: &Pubkey,
    owner_reward_account: &Pubkey,
) -> Instruction {
    build(
        vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(farm_address(collection), false),
            AccountMeta::new_readonly(*nft_mint, false),
            AccountMeta::new(*owner_nft_account, false),
            AccountMeta::new(vault_address(nft_mint), false),
            AccountMeta::new(record_address(nft_mint), false),
            AccountMeta::new(*reward_mint, false),
            AccountMeta::new(*owner_reward_account, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        NftStakingInstruction::Unstake,
    )
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match NftStakingInstruction::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        NftStakingInstruction::InitFarm { rate } => init(program_id, accounts, rate),
        NftStakingInstruction::Stake => stake_nft(program_id, accounts),
        NftStakingInstruction::Unstake => unstake_nft(program_id, accounts),
    }
}

fn save<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    let bytes = borsh::to_vec(state).map_err(|_| ProgramError::InvalidAccountData)?;
    info.try_borrow_mut_data()?[..bytes.len()].copy_from_slice(&bytes);
    Ok(())
}

fn load<T: BorshDeserialize>(program_id: &Pubkey, info: &AccountInfo) -> Result<T, ProgramError> {
    if info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    T::deserialize(&mut &info.try_borrow_data()?[..]).map_err(|_| ProgramError::InvalidAccountData)
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo], rate: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let admin = next_account_info(iter)?;
    let farm = next_account_info(iter)?;
    let collection = next_account_info(iter)?;
    let reward_mint = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    require_signer(admin)?;
    let bump = require_pda(farm, &[b"farm", collection.key.as_ref()], program_id)?;
    if mint(reward_mint)?.mint_authority != COption::Some(*farm.key) {
        return Err(NftStakingError::InvalidRewardMint.into());
    }
    create_pda_account(
        admin,
        farm,
        system_program,
        Farm::SPACE,
        program_id,
        &[b"farm", collection.key.as_ref(), &[bump]],
    )?;
    save(
        &Farm {
            collection: *collection.key,
            reward_mint: *reward_mint.key,
            rate,
            bump,
        },
        farm,
    )
}

fn stake_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let farm = next_account_info(iter)?;
    let nft_mint = next_account_info(iter)?;
    let metadata = next_account_info(iter)?;
    let owner_nft = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let record = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(owner)?;
    let farm_state: Farm = load(program_id, farm)?;
    // The metadata must be the real one for this mint (owned by the metadata
    // program, at its PDA) and name the farm's collection as *verified*.
    let meta = ex02_metadata::load(&ex02_metadata::ID, metadata)?;
    if meta.mint != *nft_mint.key {
        return Err(NftStakingError::WrongMetadata.into());
    }
    match meta.collection {
        Some(c) if c.verified && c.key == farm_state.collection => {}
        _ => return Err(NftStakingError::NotInCollection.into()),
    }
    let held = token_account(owner_nft)?;
    if held.mint != *nft_mint.key || held.owner != *owner.key || held.amount != 1 {
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_bump = require_pda(vault, &[b"nft_vault", nft_mint.key.as_ref()], program_id)?;
    let record_bump = require_pda(record, &[b"stake", nft_mint.key.as_ref()], program_id)?;
    create_token_account(
        owner,
        vault,
        nft_mint,
        farm.key,
        system_program,
        token_program,
        &[b"nft_vault", nft_mint.key.as_ref(), &[vault_bump]],
    )?;
    token_transfer(token_program, owner_nft, vault, owner, 1, &[])?;
    create_pda_account(
        owner,
        record,
        system_program,
        StakeRecord::SPACE,
        program_id,
        &[b"stake", nft_mint.key.as_ref(), &[record_bump]],
    )?;
    let state = StakeRecord {
        owner: *owner.key,
        mint: *nft_mint.key,
        staked_at: Clock::get()?.unix_timestamp,
        bump: record_bump,
    };
    save(&state, record)
}

fn unstake_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let farm = next_account_info(iter)?;
    let nft_mint = next_account_info(iter)?;
    let owner_nft = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let record = next_account_info(iter)?;
    let reward_mint = next_account_info(iter)?;
    let owner_rewards = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(owner)?;
    let farm_state: Farm = load(program_id, farm)?;
    let state: StakeRecord = load(program_id, record)?;
    let expected_record = Pubkey::create_program_address(
        &[b"stake", nft_mint.key.as_ref(), &[state.bump]],
        program_id,
    )?;
    if *record.key != expected_record || state.mint != *nft_mint.key {
        return Err(ProgramError::InvalidSeeds);
    }
    if state.owner != *owner.key {
        return Err(NftStakingError::NotTheStaker.into());
    }
    if *reward_mint.key != farm_state.reward_mint {
        return Err(NftStakingError::InvalidRewardMint.into());
    }
    let seconds = Clock::get()?
        .unix_timestamp
        .saturating_sub(state.staked_at)
        .max(0) as u64;
    let reward = seconds
        .checked_mul(farm_state.rate)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let farm_seeds: &[&[u8]] = &[b"farm", farm_state.collection.as_ref(), &[farm_state.bump]];
    token_transfer(token_program, vault, owner_nft, farm, 1, &[farm_seeds])?;
    close_token_account(token_program, vault, owner, farm, &[farm_seeds])?;
    if reward > 0 {
        token_mint_to(
            token_program,
            reward_mint,
            owner_rewards,
            farm,
            reward,
            &[farm_seeds],
        )?;
    }
    close_account(record, owner)
}
