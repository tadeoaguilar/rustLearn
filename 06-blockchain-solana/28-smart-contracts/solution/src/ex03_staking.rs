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
        ProgramError::Custom(e as u32)
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
    let elapsed = now.saturating_sub(pool.last_update).max(0) as u128;
    if pool.total_staked > 0 && elapsed > 0 {
        let increase = elapsed
            .checked_mul(pool.reward_rate as u128)
            .and_then(|r| r.checked_mul(PRECISION))
            .ok_or(StakingError::MathOverflow)?
            / pool.total_staked as u128;
        pool.reward_per_token = pool
            .reward_per_token
            .checked_add(increase)
            .ok_or(StakingError::MathOverflow)?;
    }
    pool.last_update = now.max(pool.last_update);
    Ok(())
}

/// Credit `stake` with what it earned since it was last settled.
pub fn settle(stake: &mut StakeAccount, reward_per_token: u128) -> Result<(), StakingError> {
    let delta = reward_per_token.saturating_sub(stake.reward_per_token_paid);
    let earned = (stake.amount as u128)
        .checked_mul(delta)
        .ok_or(StakingError::MathOverflow)?
        / PRECISION;
    let earned = u64::try_from(earned).map_err(|_| StakingError::MathOverflow)?;
    stake.pending = stake
        .pending
        .checked_add(earned)
        .ok_or(StakingError::MathOverflow)?;
    stake.reward_per_token_paid = reward_per_token;
    Ok(())
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
    match StakingInstruction::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        StakingInstruction::InitPool { reward_rate } => init(program_id, accounts, reward_rate),
        StakingInstruction::Stake { amount } => deposit(program_id, accounts, amount),
        StakingInstruction::Unstake { amount } => withdraw(program_id, accounts, amount),
        StakingInstruction::Claim => claim_rewards(program_id, accounts),
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

fn load<T: BorshDeserialize>(program_id: &Pubkey, info: &AccountInfo) -> Result<T, ProgramError> {
    if info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    T::deserialize(&mut &info.try_borrow_data()?[..]).map_err(|_| ProgramError::InvalidAccountData)
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo], reward_rate: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let admin = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let stake_mint = next_account_info(iter)?;
    let reward_mint = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(admin)?;
    mint(stake_mint)?;
    let bump = require_pda(pool, &[b"pool", stake_mint.key.as_ref()], program_id)?;
    let vault_bump = require_pda(vault, &[b"stake_vault", pool.key.as_ref()], program_id)?;
    // The pool mints the rewards, so it must be the reward mint's authority.
    if mint(reward_mint)?.mint_authority != COption::Some(*pool.key) {
        return Err(StakingError::InvalidRewardMint.into());
    }
    create_pda_account(
        admin,
        pool,
        system_program,
        Pool::SPACE,
        program_id,
        &[b"pool", stake_mint.key.as_ref(), &[bump]],
    )?;
    create_token_account(
        admin,
        vault,
        stake_mint,
        pool.key,
        system_program,
        token_program,
        &[b"stake_vault", pool.key.as_ref(), &[vault_bump]],
    )?;
    let state = Pool {
        admin: *admin.key,
        stake_mint: *stake_mint.key,
        reward_mint: *reward_mint.key,
        reward_rate,
        total_staked: 0,
        reward_per_token: 0,
        last_update: Clock::get()?.unix_timestamp,
        bump,
        vault_bump,
    };
    save(&state, pool)
}

/// The pool, brought up to date, and the vault checked against it.
fn load_pool(
    program_id: &Pubkey,
    pool: &AccountInfo,
    vault: Option<&AccountInfo>,
) -> Result<Pool, ProgramError> {
    let mut state: Pool = load(program_id, pool)?;
    if let Some(vault) = vault {
        let expected = Pubkey::create_program_address(
            &[b"stake_vault", pool.key.as_ref(), &[state.vault_bump]],
            program_id,
        )?;
        if *vault.key != expected {
            return Err(ProgramError::InvalidSeeds);
        }
    }
    accrue(&mut state, Clock::get()?.unix_timestamp)?;
    Ok(state)
}

/// The user's stake account, settled against the pool.
fn load_stake(
    program_id: &Pubkey,
    info: &AccountInfo,
    pool: &AccountInfo,
    user: &Pubkey,
    state: &Pool,
) -> Result<StakeAccount, ProgramError> {
    let mut stake: StakeAccount = load(program_id, info)?;
    if stake.owner != *user || stake.pool != *pool.key {
        return Err(StakingError::NotTheStaker.into());
    }
    settle(&mut stake, state.reward_per_token)?;
    Ok(stake)
}

fn pool_seeds(state: &Pool) -> [Vec<u8>; 3] {
    [
        b"pool".to_vec(),
        state.stake_mint.to_bytes().to_vec(),
        vec![state.bump],
    ]
}

fn deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let stake_info = next_account_info(iter)?;
    let user_tokens = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(user)?;
    if amount == 0 {
        return Err(StakingError::ZeroAmount.into());
    }
    let mut state = load_pool(program_id, pool, Some(vault))?;
    if token_account(user_tokens)?.mint != state.stake_mint {
        return Err(StakingError::WrongMint.into());
    }

    // First stake: create the stake account.
    if stake_info.data_is_empty() {
        let bump = require_pda(
            stake_info,
            &[b"stake", pool.key.as_ref(), user.key.as_ref()],
            program_id,
        )?;
        create_pda_account(
            user,
            stake_info,
            system_program,
            StakeAccount::SPACE,
            program_id,
            &[b"stake", pool.key.as_ref(), user.key.as_ref(), &[bump]],
        )?;
        let fresh = StakeAccount {
            pool: *pool.key,
            owner: *user.key,
            amount: 0,
            reward_per_token_paid: state.reward_per_token,
            pending: 0,
            bump,
        };
        save(&fresh, stake_info)?;
    }
    let mut stake = load_stake(program_id, stake_info, pool, user.key, &state)?;

    token_transfer(token_program, user_tokens, vault, user, amount, &[])?;
    stake.amount = stake
        .amount
        .checked_add(amount)
        .ok_or(StakingError::MathOverflow)?;
    state.total_staked = state
        .total_staked
        .checked_add(amount)
        .ok_or(StakingError::MathOverflow)?;
    save(&stake, stake_info)?;
    save(&state, pool)
}

fn withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let stake_info = next_account_info(iter)?;
    let user_tokens = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(user)?;
    if amount == 0 {
        return Err(StakingError::ZeroAmount.into());
    }
    let mut state = load_pool(program_id, pool, Some(vault))?;
    let mut stake = load_stake(program_id, stake_info, pool, user.key, &state)?;
    stake.amount = stake
        .amount
        .checked_sub(amount)
        .ok_or(StakingError::InsufficientStake)?;
    state.total_staked = state
        .total_staked
        .checked_sub(amount)
        .ok_or(StakingError::MathOverflow)?;
    save(&stake, stake_info)?;
    save(&state, pool)?;

    let seeds = pool_seeds(&state);
    let seeds: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
    token_transfer(token_program, vault, user_tokens, pool, amount, &[&seeds])
}

fn claim_rewards(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let stake_info = next_account_info(iter)?;
    let reward_mint = next_account_info(iter)?;
    let user_rewards = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(user)?;
    let state = load_pool(program_id, pool, None)?;
    if *reward_mint.key != state.reward_mint {
        return Err(StakingError::WrongMint.into());
    }
    let mut stake = load_stake(program_id, stake_info, pool, user.key, &state)?;
    let amount = std::mem::take(&mut stake.pending);
    save(&stake, stake_info)?;
    save(&state, pool)?;
    if amount > 0 {
        let seeds = pool_seeds(&state);
        let seeds: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
        token_mint_to(
            token_program,
            reward_mint,
            user_rewards,
            pool,
            amount,
            &[&seeds],
        )?;
    }
    Ok(())
}
