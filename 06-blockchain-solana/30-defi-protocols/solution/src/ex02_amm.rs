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
        ProgramError::Custom(e as u32)
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
    match AmmInstruction::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)? {
        AmmInstruction::InitPool { fee_bps } => init(program_id, accounts, fee_bps),
        AmmInstruction::AddLiquidity {
            max_a,
            max_b,
            min_shares,
        } => add(program_id, accounts, max_a, max_b, min_shares),
        AmmInstruction::RemoveLiquidity {
            shares,
            min_a,
            min_b,
        } => remove(program_id, accounts, shares, min_a, min_b),
        AmmInstruction::Swap {
            amount_in,
            min_out,
            a_to_b,
        } => swap_tokens(program_id, accounts, amount_in, min_out, a_to_b),
    }
}

fn save(state: &Pool, info: &AccountInfo) -> ProgramResult {
    let bytes = borsh::to_vec(state).map_err(|_| ProgramError::InvalidAccountData)?;
    info.try_borrow_mut_data()?[..bytes.len()].copy_from_slice(&bytes);
    Ok(())
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo], fee_bps: u64) -> ProgramResult {
    let iter = &mut accounts.iter();
    let payer = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let mint_a = next_account_info(iter)?;
    let mint_b = next_account_info(iter)?;
    let vault_a = next_account_info(iter)?;
    let vault_b = next_account_info(iter)?;
    let lp_mint = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(payer)?;
    check_token_program(token_program)?;
    if mint_a.key >= mint_b.key {
        return Err(AmmError::UnorderedMints.into());
    }
    if fee_bps >= 1_000 {
        return Err(AmmError::BadFee.into());
    }
    let bump = require_pda(
        pool,
        &[b"pool", mint_a.key.as_ref(), mint_b.key.as_ref()],
        program_id,
    )?;
    let vault_a_bump = require_pda(vault_a, &[b"vault_a", pool.key.as_ref()], program_id)?;
    let vault_b_bump = require_pda(vault_b, &[b"vault_b", pool.key.as_ref()], program_id)?;
    let lp_bump = require_pda(lp_mint, &[b"lp", pool.key.as_ref()], program_id)?;

    create_pda_account(
        payer,
        pool,
        system_program,
        Pool::SPACE,
        program_id,
        &[b"pool", mint_a.key.as_ref(), mint_b.key.as_ref(), &[bump]],
    )?;
    create_token_account(
        payer,
        vault_a,
        mint_a,
        pool.key,
        system_program,
        token_program,
        &[b"vault_a", pool.key.as_ref(), &[vault_a_bump]],
    )?;
    create_token_account(
        payer,
        vault_b,
        mint_b,
        pool.key,
        system_program,
        token_program,
        &[b"vault_b", pool.key.as_ref(), &[vault_b_bump]],
    )?;
    // The LP mint: 6 decimals, the pool mints and nobody freezes.
    create_pda_account(
        payer,
        lp_mint,
        system_program,
        spl_token_interface::state::Mint::LEN,
        &spl_token_interface::ID,
        &[b"lp", pool.key.as_ref(), &[lp_bump]],
    )?;
    invoke_signed(
        &spl_token_interface::instruction::initialize_mint2(
            &spl_token_interface::ID,
            lp_mint.key,
            pool.key,
            None,
            6,
        )?,
        &[lp_mint.clone(), token_program.clone()],
        &[],
    )?;
    let state = Pool {
        mint_a: *mint_a.key,
        mint_b: *mint_b.key,
        fee_bps,
        reserve_a: 0,
        reserve_b: 0,
        total_shares: 0,
        bump,
        vault_a_bump,
        vault_b_bump,
        lp_bump,
    };
    save(&state, pool)
}

/// The pool's state, after checking the vaults (and LP mint) are its own.
fn load_pool(
    program_id: &Pubkey,
    pool: &AccountInfo,
    vault_a: &AccountInfo,
    vault_b: &AccountInfo,
    lp_mint: Option<&AccountInfo>,
) -> Result<Pool, ProgramError> {
    if pool.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let state = Pool::deserialize(&mut &pool.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let check = |info: &AccountInfo, seed: &[u8], bump: u8| -> ProgramResult {
        let expected =
            Pubkey::create_program_address(&[seed, pool.key.as_ref(), &[bump]], program_id)?;
        if *info.key != expected {
            return Err(AmmError::WrongPoolAccount.into());
        }
        Ok(())
    };
    check(vault_a, b"vault_a", state.vault_a_bump)?;
    check(vault_b, b"vault_b", state.vault_b_bump)?;
    if let Some(lp) = lp_mint {
        check(lp, b"lp", state.lp_bump)?;
    }
    Ok(state)
}

fn pool_seeds(state: &Pool) -> [Vec<u8>; 4] {
    [
        b"pool".to_vec(),
        state.mint_a.to_bytes().to_vec(),
        state.mint_b.to_bytes().to_vec(),
        vec![state.bump],
    ]
}

fn check_mint(info: &AccountInfo, mint: &Pubkey) -> ProgramResult {
    if token_account(info)?.mint != *mint {
        return Err(AmmError::WrongMint.into());
    }
    Ok(())
}

fn add(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_a: u64,
    max_b: u64,
    min_shares: u64,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let vault_a = next_account_info(iter)?;
    let vault_b = next_account_info(iter)?;
    let lp_mint = next_account_info(iter)?;
    let user_a = next_account_info(iter)?;
    let user_b = next_account_info(iter)?;
    let user_lp = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(user)?;
    let mut state = load_pool(program_id, pool, vault_a, vault_b, Some(lp_mint))?;
    check_mint(user_a, &state.mint_a)?;
    check_mint(user_b, &state.mint_b)?;

    let (shares, amount_a, amount_b, minted_total) = if state.total_shares == 0 {
        let (user_shares, locked) =
            math::initial_shares(max_a, max_b).ok_or(AmmError::InsufficientLiquidity)?;
        (user_shares, max_a, max_b, user_shares + locked)
    } else {
        let (shares, a, b) = math::deposit(
            max_a,
            max_b,
            state.reserve_a,
            state.reserve_b,
            state.total_shares,
        )
        .ok_or(AmmError::InsufficientLiquidity)?;
        (shares, a, b, shares)
    };
    if shares < min_shares || amount_a > max_a || amount_b > max_b {
        return Err(AmmError::SlippageExceeded.into());
    }

    state.reserve_a = state
        .reserve_a
        .checked_add(amount_a)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    state.reserve_b = state
        .reserve_b
        .checked_add(amount_b)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    state.total_shares = state
        .total_shares
        .checked_add(minted_total)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    save(&state, pool)?;

    token_transfer(token_program, user_a, vault_a, user, amount_a, &[])?;
    token_transfer(token_program, user_b, vault_b, user, amount_b, &[])?;
    let seeds = pool_seeds(&state);
    let seeds: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
    token_mint_to(token_program, lp_mint, user_lp, pool, shares, &[&seeds])
}

fn remove(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    shares: u64,
    min_a: u64,
    min_b: u64,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let vault_a = next_account_info(iter)?;
    let vault_b = next_account_info(iter)?;
    let lp_mint = next_account_info(iter)?;
    let user_a = next_account_info(iter)?;
    let user_b = next_account_info(iter)?;
    let user_lp = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(user)?;
    if shares == 0 {
        return Err(AmmError::ZeroAmount.into());
    }
    let mut state = load_pool(program_id, pool, vault_a, vault_b, Some(lp_mint))?;
    check_mint(user_a, &state.mint_a)?;
    check_mint(user_b, &state.mint_b)?;
    let (amount_a, amount_b) =
        math::withdraw(shares, state.reserve_a, state.reserve_b, state.total_shares)
            .ok_or(AmmError::InsufficientLiquidity)?;
    if amount_a < min_a || amount_b < min_b {
        return Err(AmmError::SlippageExceeded.into());
    }
    state.reserve_a -= amount_a;
    state.reserve_b -= amount_b;
    state.total_shares -= shares;
    save(&state, pool)?;

    // Burning fails if the user doesn't hold the shares.
    token_burn(token_program, user_lp, lp_mint, user, shares)?;
    let seeds = pool_seeds(&state);
    let seeds: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
    token_transfer(token_program, vault_a, user_a, pool, amount_a, &[&seeds])?;
    token_transfer(token_program, vault_b, user_b, pool, amount_b, &[&seeds])
}

fn swap_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_in: u64,
    min_out: u64,
    a_to_b: bool,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let user = next_account_info(iter)?;
    let pool = next_account_info(iter)?;
    let vault_a = next_account_info(iter)?;
    let vault_b = next_account_info(iter)?;
    let source = next_account_info(iter)?;
    let destination = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    require_signer(user)?;
    if amount_in == 0 {
        return Err(AmmError::ZeroAmount.into());
    }
    let mut state = load_pool(program_id, pool, vault_a, vault_b, None)?;
    let (vault_in, vault_out, mint_in, mint_out, reserve_in, reserve_out) = if a_to_b {
        (
            vault_a,
            vault_b,
            state.mint_a,
            state.mint_b,
            state.reserve_a,
            state.reserve_b,
        )
    } else {
        (
            vault_b,
            vault_a,
            state.mint_b,
            state.mint_a,
            state.reserve_b,
            state.reserve_a,
        )
    };
    check_mint(source, &mint_in)?;
    check_mint(destination, &mint_out)?;
    let out = math::swap_out(amount_in, reserve_in, reserve_out, state.fee_bps)
        .ok_or(AmmError::InsufficientLiquidity)?;
    if out == 0 || out >= reserve_out {
        return Err(AmmError::InsufficientLiquidity.into());
    }
    if out < min_out {
        return Err(AmmError::SlippageExceeded.into());
    }
    let (new_in, new_out) = (
        reserve_in
            .checked_add(amount_in)
            .ok_or(ProgramError::ArithmeticOverflow)?,
        reserve_out - out,
    );
    if a_to_b {
        (state.reserve_a, state.reserve_b) = (new_in, new_out);
    } else {
        (state.reserve_b, state.reserve_a) = (new_in, new_out);
    }
    save(&state, pool)?;

    token_transfer(token_program, source, vault_in, user, amount_in, &[])?;
    let seeds = pool_seeds(&state);
    let seeds: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
    token_transfer(token_program, vault_out, destination, pool, out, &[&seeds])
}
