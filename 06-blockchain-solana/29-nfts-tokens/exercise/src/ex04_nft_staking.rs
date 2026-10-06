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
        todo!("Exercise 4")
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
    todo!("Exercise 4")
}

fn save<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    todo!("Exercise 4")
}

fn load<T: BorshDeserialize>(program_id: &Pubkey, info: &AccountInfo) -> Result<T, ProgramError> {
    todo!("Exercise 4")
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo], rate: u64) -> ProgramResult {
    todo!("Exercise 4")
}

fn stake_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    todo!("Exercise 4")
}

fn unstake_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    todo!("Exercise 4")
}
