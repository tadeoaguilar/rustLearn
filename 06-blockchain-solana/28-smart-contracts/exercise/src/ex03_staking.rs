//! Exercise 3: a staking pool with continuous rewards.
//!
//! Users stake token S into a vault and earn token R, minted by the pool at
//! `reward_rate` base units per second, shared in proportion to stake. Paying
//! everyone every second is impossible, so the pool keeps one number: the
//! rewards one staked token has earned since the start, `reward_per_token`
//! (scaled by `PRECISION`). A user's rewards are
//! `amount * (reward_per_token - reward_per_token_paid)`, settled whenever
//! their stake changes. It's the accumulator behind Synthetix's
//! StakingRewards and most farming contracts: O(1) per action, any number of
//! stakers.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

use crate::util::{
    create_pda_account, create_token_account, mint, require_pda, require_signer, token_account,
    token_mint_to, token_transfer,
};

solana_program::declare_id!("8Kq7WChVDSFxH55nNz9LZqa9tK1PgtXLDsBV8jkGQaL4");

/// `reward_per_token` is stored multiplied by this, so small rewards spread
/// over a large stake don't round down to zero.
pub const PRECISION: u128 = 1_000_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum StakingError {
    #[error("amounts must be greater than zero")]
    ZeroAmount = 0,
    #[error("unstaking more than is staked")]
    InsufficientStake = 1,
    #[error("the reward mint's mint authority must be the pool")]
    InvalidRewardMint = 2,
    #[error("a token account has the wrong mint")]
    WrongMint = 3,
    #[error("the stake account belongs to someone else")]
    NotTheStaker = 4,
    #[error("arithmetic overflow")]
    MathOverflow = 5,
}

impl From<StakingError> for ProgramError {
    fn from(e: StakingError) -> Self {
        todo!("Exercise 3")
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Pool {
    pub admin: Pubkey,
    pub stake_mint: Pubkey,
    pub reward_mint: Pubkey,
    /// Reward base units minted per second, for the whole pool.
    pub reward_rate: u64,
    pub total_staked: u64,
    /// Rewards per staked base unit since the start, times `PRECISION`.
    pub reward_per_token: u128,
    pub last_update: i64,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Pool {
    pub const SPACE: usize = 32 * 3 + 8 + 8 + 16 + 8 + 1 + 1;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct StakeAccount {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    /// The pool's `reward_per_token` when this account was last settled.
    pub reward_per_token_paid: u128,
    /// Rewards settled but not yet claimed.
    pub pending: u64,
    pub bump: u8,
}

impl StakeAccount {
    pub const SPACE: usize = 32 + 32 + 8 + 16 + 8 + 1;
}

// -------------------------------------------------------------------- math

/// Bring `reward_per_token` up to `now`. With nothing staked, no rewards
/// accrue (and none are owed to anyone).
pub fn accrue(pool: &mut Pool, now: i64) -> Result<(), StakingError> {
    todo!("Exercise 3")
}

/// Credit `stake` with what it earned since it was last settled.
pub fn settle(stake: &mut StakeAccount, reward_per_token: u128) -> Result<(), StakingError> {
    todo!("Exercise 3")
}

// ------------------------------------------------------------------ client

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum StakingInstruction {
    /// `[admin (s, w), pool (w), stake mint, reward mint, vault (w), system program, token program]`
    InitPool { reward_rate: u64 },
    /// `[user (s, w), pool (w), stake account (w), user's S account (w), vault (w), system program, token program]`
    Stake { amount: u64 },
    /// `[user (s), pool (w), stake account (w), user's S account (w), vault (w), token program]`
    Unstake { amount: u64 },
    /// `[user (s), pool (w), stake account (w), reward mint (w), user's R account (w), token program]`
    Claim,
}

pub fn pool_address(stake_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"pool", stake_mint.as_ref()], &ID).0
}

pub fn vault_address(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"stake_vault", pool.as_ref()], &ID).0
}

pub fn stake_account_address(pool: &Pubkey, user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"stake", pool.as_ref(), user.as_ref()], &ID).0
}

fn build(accounts: Vec<AccountMeta>, data: StakingInstruction) -> Instruction {
    Instruction {
        program_id: ID,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

pub fn init_pool(
    admin: &Pubkey,
    stake_mint: &Pubkey,
    reward_mint: &Pubkey,
    reward_rate: u64,
) -> Instruction {
    let pool = pool_address(stake_mint);
    build(
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(pool, false),
            AccountMeta::new_readonly(*stake_mint, false),
            AccountMeta::new_readonly(*reward_mint, false),
            AccountMeta::new(vault_address(&pool), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        StakingInstruction::InitPool { reward_rate },
    )
}

pub fn stake(user: &Pubkey, stake_mint: &Pubkey, user_tokens: &Pubkey, amount: u64) -> Instruction {
    let pool = pool_address(stake_mint);
    build(
        vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(pool, false),
            AccountMeta::new(stake_account_address(&pool, user), false),
            AccountMeta::new(*user_tokens, false),
            AccountMeta::new(vault_address(&pool), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        StakingInstruction::Stake { amount },
    )
}

pub fn unstake(
    user: &Pubkey,
    stake_mint: &Pubkey,
    user_tokens: &Pubkey,
    amount: u64,
) -> Instruction {
    let pool = pool_address(stake_mint);
    build(
        vec![
            AccountMeta::new_readonly(*user, true),
            AccountMeta::new(pool, false),
            AccountMeta::new(stake_account_address(&pool, user), false),
            AccountMeta::new(*user_tokens, false),
            AccountMeta::new(vault_address(&pool), false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        StakingInstruction::Unstake { amount },
    )
}

pub fn claim(
    user: &Pubkey,
    stake_mint: &Pubkey,
    reward_mint: &Pubkey,
    user_rewards: &Pubkey,
) -> Instruction {
    let pool = pool_address(stake_mint);
    build(
        vec![
            AccountMeta::new_readonly(*user, true),
            AccountMeta::new(pool, false),
            AccountMeta::new(stake_account_address(&pool, user), false),
            AccountMeta::new(*reward_mint, false),
            AccountMeta::new(*user_rewards, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        StakingInstruction::Claim,
    )
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    todo!("Exercise 3")
}

fn save<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    todo!("Exercise 3")
}

fn load<T: BorshDeserialize>(program_id: &Pubkey, info: &AccountInfo) -> Result<T, ProgramError> {
    todo!("Exercise 3")
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo], reward_rate: u64) -> ProgramResult {
    todo!("Exercise 3")
}

/// The pool, brought up to date, and the vault checked against it.
fn load_pool(
    program_id: &Pubkey,
    pool: &AccountInfo,
    vault: Option<&AccountInfo>,
) -> Result<Pool, ProgramError> {
    todo!("Exercise 3")
}

/// The user's stake account, settled against the pool.
fn load_stake(
    program_id: &Pubkey,
    info: &AccountInfo,
    pool: &AccountInfo,
    user: &Pubkey,
    state: &Pool,
) -> Result<StakeAccount, ProgramError> {
    todo!("Exercise 3")
}

fn pool_seeds(state: &Pool) -> [Vec<u8>; 3] {
    todo!("Exercise 3")
}

fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    todo!("Exercise 3")
}

fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    todo!("Exercise 3")
}

fn claim_rewards(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    todo!("Exercise 3")
}
