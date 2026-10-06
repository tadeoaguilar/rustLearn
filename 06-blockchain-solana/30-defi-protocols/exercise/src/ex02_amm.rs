//! Exercise 2: the AMM program.
//!
//! One pool per pair of mints (ordered, so A/B and B/A are the same pool):
//! PDA `["pool", mint_a, mint_b]`, two vaults it owns (`["vault_a", pool]`,
//! `["vault_b", pool]`) and an LP mint whose authority is the pool
//! (`["lp", pool]`). The pool *stores* its reserves rather than reading the
//! vault balances: anyone can send tokens to a vault, and the maths must not
//! change because someone did.
//!
//! Every instruction that moves value takes a limit from the user
//! (`min_out`, `min_shares`, `min_a`/`min_b`): the price can move between
//! signing and execution, and without a limit the user accepts any price.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use crate::ex01_amm_math as math;
use crate::util::{
    check_token_program, create_pda_account, create_token_account, require_pda, require_signer,
    token_account, token_burn, token_mint_to, token_transfer,
};

solana_program::declare_id!("DykumXnqtDGemejR8j5bfnybd8d2gsmjT63ENmDfHxqb");

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum AmmError {
    #[error("mint A must sort before mint B")]
    UnorderedMints = 0,
    #[error("the fee must be below 10%")]
    BadFee = 1,
    #[error("the price moved past your limit")]
    SlippageExceeded = 2,
    #[error("too little liquidity for this")]
    InsufficientLiquidity = 3,
    #[error("a token account has the wrong mint")]
    WrongMint = 4,
    #[error("a vault or LP mint doesn't belong to this pool")]
    WrongPoolAccount = 5,
    #[error("amounts must be greater than zero")]
    ZeroAmount = 6,
}

impl From<AmmError> for ProgramError {
    fn from(e: AmmError) -> Self {
        todo!("Exercise 2")
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Pool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub fee_bps: u64,
    pub reserve_a: u64,
    pub reserve_b: u64,
    /// LP shares in existence: the LP mint's supply plus `MINIMUM_LIQUIDITY`
    /// locked at the first deposit.
    pub total_shares: u64,
    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
    pub lp_bump: u8,
}

impl Pool {
    pub const SPACE: usize = 32 + 32 + 8 * 4 + 4;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum AmmInstruction {
    /// `[payer (s, w), pool (w), mint A, mint B, vault A (w), vault B (w), LP mint (w), system, token]`
    InitPool { fee_bps: u64 },
    /// `[user (s), pool (w), vault A (w), vault B (w), LP mint (w), user A (w), user B (w), user LP (w), token]`
    AddLiquidity {
        max_a: u64,
        max_b: u64,
        min_shares: u64,
    },
    /// Same accounts as `AddLiquidity`.
    RemoveLiquidity { shares: u64, min_a: u64, min_b: u64 },
    /// `[user (s), pool (w), vault A (w), vault B (w), user source (w), user destination (w), token]`
    Swap {
        amount_in: u64,
        min_out: u64,
        a_to_b: bool,
    },
}

// ------------------------------------------------------------------ client

pub fn pool_address(mint_a: &Pubkey, mint_b: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"pool", mint_a.as_ref(), mint_b.as_ref()], &ID).0
}

pub fn vault_addresses(pool: &Pubkey) -> (Pubkey, Pubkey) {
    (
        Pubkey::find_program_address(&[b"vault_a", pool.as_ref()], &ID).0,
        Pubkey::find_program_address(&[b"vault_b", pool.as_ref()], &ID).0,
    )
}

pub fn lp_mint_address(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"lp", pool.as_ref()], &ID).0
}

/// The two mints in pool order.
pub fn sorted(x: Pubkey, y: Pubkey) -> (Pubkey, Pubkey) {
    if x < y { (x, y) } else { (y, x) }
}

fn build(accounts: Vec<AccountMeta>, data: AmmInstruction) -> Instruction {
    Instruction {
        program_id: ID,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

pub fn init_pool(payer: &Pubkey, mint_a: &Pubkey, mint_b: &Pubkey, fee_bps: u64) -> Instruction {
    let pool = pool_address(mint_a, mint_b);
    let (vault_a, vault_b) = vault_addresses(&pool);
    build(
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(pool, false),
            AccountMeta::new_readonly(*mint_a, false),
            AccountMeta::new_readonly(*mint_b, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(vault_b, false),
            AccountMeta::new(lp_mint_address(&pool), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        AmmInstruction::InitPool { fee_bps },
    )
}

fn liquidity_accounts(
    user: &Pubkey,
    pool: &Pubkey,
    user_a: &Pubkey,
    user_b: &Pubkey,
    user_lp: &Pubkey,
) -> Vec<AccountMeta> {
    let (vault_a, vault_b) = vault_addresses(pool);
    vec![
        AccountMeta::new_readonly(*user, true),
        AccountMeta::new(*pool, false),
        AccountMeta::new(vault_a, false),
        AccountMeta::new(vault_b, false),
        AccountMeta::new(lp_mint_address(pool), false),
        AccountMeta::new(*user_a, false),
        AccountMeta::new(*user_b, false),
        AccountMeta::new(*user_lp, false),
        AccountMeta::new_readonly(spl_token_interface::ID, false),
    ]
}

#[allow(clippy::too_many_arguments)] // one per account and argument, as on the wire
pub fn add_liquidity(
    user: &Pubkey,
    pool: &Pubkey,
    user_a: &Pubkey,
    user_b: &Pubkey,
    user_lp: &Pubkey,
    max_a: u64,
    max_b: u64,
    min_shares: u64,
) -> Instruction {
    build(
        liquidity_accounts(user, pool, user_a, user_b, user_lp),
        AmmInstruction::AddLiquidity {
            max_a,
            max_b,
            min_shares,
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn remove_liquidity(
    user: &Pubkey,
    pool: &Pubkey,
    user_a: &Pubkey,
    user_b: &Pubkey,
    user_lp: &Pubkey,
    shares: u64,
    min_a: u64,
    min_b: u64,
) -> Instruction {
    build(
        liquidity_accounts(user, pool, user_a, user_b, user_lp),
        AmmInstruction::RemoveLiquidity {
            shares,
            min_a,
            min_b,
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn swap(
    user: &Pubkey,
    pool: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    amount_in: u64,
    min_out: u64,
    a_to_b: bool,
) -> Instruction {
    let (vault_a, vault_b) = vault_addresses(pool);
    build(
        vec![
            AccountMeta::new_readonly(*user, true),
            AccountMeta::new(*pool, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(vault_b, false),
            AccountMeta::new(*source, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        AmmInstruction::Swap {
            amount_in,
            min_out,
            a_to_b,
        },
    )
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    todo!("Exercise 2")
}

fn save(state: &Pool, info: &AccountInfo) -> ProgramResult {
    todo!("Exercise 2")
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo], fee_bps: u64) -> ProgramResult {
    todo!("Exercise 2")
}

/// The pool's state, after checking the vaults (and LP mint) are its own.
fn load_pool(
    program_id: &Pubkey,
    pool: &AccountInfo,
    vault_a: &AccountInfo,
    vault_b: &AccountInfo,
    lp_mint: Option<&AccountInfo>,
) -> Result<Pool, ProgramError> {
    todo!("Exercise 2")
}

fn pool_seeds(state: &Pool) -> [Vec<u8>; 4] {
    todo!("Exercise 2")
}

fn check_mint(info: &AccountInfo, mint: &Pubkey) -> ProgramResult {
    todo!("Exercise 2")
}

fn add(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_a: u64,
    max_b: u64,
    min_shares: u64,
) -> ProgramResult {
    todo!("Exercise 2")
}

fn remove(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    shares: u64,
    min_a: u64,
    min_b: u64,
) -> ProgramResult {
    todo!("Exercise 2")
}

fn swap_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_in: u64,
    min_out: u64,
    a_to_b: bool,
) -> ProgramResult {
    todo!("Exercise 2")
}
