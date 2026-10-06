//! Exercise 5: a lending market.
//!
//! One market, two tokens: users deposit **collateral** (say, a SOL-like
//! token) and borrow the **borrow token** (say, a USD stablecoin) against it,
//! valued with oracle prices (Exercise 4). Interest accrues through the
//! borrow index (Exercise 3) at a rate set by utilization. When a position's
//! health factor drops below 1, anyone may liquidate it.
//!
//! Accounts: the market `["market", collateral_mint, borrow_mint]`, its two
//! vaults `["collateral", market]` and `["liquidity", market]`, and one
//! obligation per user `["obligation", market, owner]`.
//!
//! Simplification: liquidity is supplied without receiving interest-bearing
//! shares (real markets mint "cTokens"/"aTokens" to suppliers; the maths is
//! the AMM's LP shares again).

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

use crate::ex03_lending_math::{self as math, RateModel, WAD};
use crate::ex04_oracle;
use crate::util::{
    create_pda_account, create_token_account, mint, require_pda, require_signer, token_account,
    token_transfer,
};

solana_program::declare_id!("7kD7Sktnzxh9sybC6ymPkDKjyY48PY93KgzAoDYM6dDP");

/// Oracle prices older than this are refused.
pub const MAX_PRICE_AGE: i64 = 60;
/// ...and so are prices less certain than ±2%.
pub const MAX_CONF_BPS: u64 = 200;
/// Values are computed in micro-units of the quote currency.
pub const VALUE_DECIMALS: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum LendingError {
    #[error("invalid market parameters")]
    BadParams = 0,
    #[error("the borrow would exceed the loan-to-value limit")]
    ExceedsLtv = 1,
    #[error("not enough liquidity in the market")]
    InsufficientLiquidity = 2,
    #[error("the position is healthy and can't be liquidated")]
    Healthy = 3,
    #[error("withdrawing that much would leave the position under-collateralized")]
    WouldBeUnhealthy = 4,
    #[error("an account doesn't belong to this market")]
    WrongAccount = 5,
    #[error("amounts must be greater than zero")]
    ZeroAmount = 6,
    #[error("more than the obligation holds")]
    InsufficientCollateral = 7,
    #[error("arithmetic overflow")]
    MathOverflow = 8,
}

impl From<LendingError> for ProgramError {
    fn from(e: LendingError) -> Self {
        todo!("Exercise 5")
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketParams {
    /// Borrow at most this fraction of the collateral's value.
    pub ltv_bps: u64,
    /// Liquidatable once debt exceeds this fraction (> LTV, the safety margin).
    pub liquidation_threshold_bps: u64,
    /// Extra collateral a liquidator receives.
    pub liquidation_bonus_bps: u64,
    /// The share of a debt one liquidation may repay.
    pub close_factor_bps: u64,
    pub base_rate_bps: u64,
    pub slope1_bps: u64,
    pub slope2_bps: u64,
    pub kink_bps: u64,
}

impl MarketParams {
    pub fn rate_model(&self) -> RateModel {
        RateModel {
            base_bps: self.base_rate_bps,
            slope1_bps: self.slope1_bps,
            slope2_bps: self.slope2_bps,
            kink_bps: self.kink_bps,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Market {
    pub admin: Pubkey,
    pub collateral_mint: Pubkey,
    pub borrow_mint: Pubkey,
    pub collateral_feed: Pubkey,
    pub borrow_feed: Pubkey,
    pub collateral_decimals: u8,
    pub borrow_decimals: u8,
    pub params: MarketParams,
    /// Borrow tokens supplied (lent out or not).
    pub total_supplied: u64,
    /// Sum of every obligation's scaled debt.
    pub total_debt_scaled: u128,
    pub borrow_index: u128,
    pub last_update: i64,
    pub bump: u8,
    pub collateral_vault_bump: u8,
    pub liquidity_vault_bump: u8,
}

impl Market {
    pub const SPACE: usize = 32 * 5 + 1 + 1 + 8 * 8 + 8 + 16 + 16 + 8 + 3;

    /// Bring the borrow index up to `now`, at the rate the utilization implies.
    pub fn accrue(&mut self, now: i64) -> Result<(), LendingError> {
        todo!("Exercise 5")
    }

    pub fn total_debt(&self) -> Option<u64> {
        todo!("Exercise 5")
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Obligation {
    pub market: Pubkey,
    pub owner: Pubkey,
    pub collateral: u64,
    /// Debt divided by the borrow index when it was taken (WAD-scaled).
    pub debt_scaled: u128,
    pub bump: u8,
}

impl Obligation {
    pub const SPACE: usize = 32 + 32 + 8 + 16 + 1;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum LendingInstruction {
    /// `[admin (s, w), market (w), collateral mint, borrow mint, collateral feed, borrow feed,
    ///   collateral vault (w), liquidity vault (w), system, token]`
    InitMarket { params: MarketParams },
    /// `[supplier (s), market (w), liquidity vault (w), supplier's borrow-token account (w), token]`
    Supply { amount: u64 },
    /// `[owner (s, w), market (w), obligation (w), collateral vault (w), owner's collateral account (w), system, token]`
    Deposit { amount: u64 },
    /// `[owner (s), market (w), obligation (w), liquidity vault (w), owner's borrow-token account (w),
    ///   collateral feed, borrow feed, token]`
    Borrow { amount: u64 },
    /// `[payer (s), market (w), obligation (w), liquidity vault (w), payer's borrow-token account (w), token]`
    Repay { amount: u64 },
    /// `[owner (s), market (w), obligation (w), collateral vault (w), owner's collateral account (w),
    ///   collateral feed, borrow feed, token]`
    Withdraw { amount: u64 },
    /// `[liquidator (s), market (w), obligation (w), liquidity vault (w), collateral vault (w),
    ///   liquidator's borrow-token account (w), liquidator's collateral account (w), collateral feed, borrow feed, token]`
    Liquidate { max_repay: u64 },
}

// ------------------------------------------------------------------ client

pub fn market_address(collateral_mint: &Pubkey, borrow_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"market", collateral_mint.as_ref(), borrow_mint.as_ref()],
        &ID,
    )
    .0
}

pub fn vault_addresses(market: &Pubkey) -> (Pubkey, Pubkey) {
    (
        Pubkey::find_program_address(&[b"collateral", market.as_ref()], &ID).0,
        Pubkey::find_program_address(&[b"liquidity", market.as_ref()], &ID).0,
    )
}

pub fn obligation_address(market: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"obligation", market.as_ref(), owner.as_ref()], &ID).0
}

/// The addresses a client needs for one market.
#[derive(Debug, Clone, Copy)]
pub struct MarketKeys {
    pub market: Pubkey,
    pub collateral_vault: Pubkey,
    pub liquidity_vault: Pubkey,
    pub collateral_feed: Pubkey,
    pub borrow_feed: Pubkey,
}

impl MarketKeys {
    pub fn new(
        collateral_mint: &Pubkey,
        borrow_mint: &Pubkey,
        collateral_feed: &Pubkey,
        borrow_feed: &Pubkey,
    ) -> Self {
        let market = market_address(collateral_mint, borrow_mint);
        let (collateral_vault, liquidity_vault) = vault_addresses(&market);
        MarketKeys {
            market,
            collateral_vault,
            liquidity_vault,
            collateral_feed: *collateral_feed,
            borrow_feed: *borrow_feed,
        }
    }
}

fn build(accounts: Vec<AccountMeta>, data: LendingInstruction) -> Instruction {
    Instruction {
        program_id: ID,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

const TOKEN: AccountMeta = AccountMeta {
    pubkey: spl_token_interface::ID,
    is_signer: false,
    is_writable: false,
};
const SYSTEM: AccountMeta = AccountMeta {
    pubkey: solana_system_interface::program::ID,
    is_signer: false,
    is_writable: false,
};

pub fn init_market(
    admin: &Pubkey,
    collateral_mint: &Pubkey,
    borrow_mint: &Pubkey,
    k: &MarketKeys,
    params: MarketParams,
) -> Instruction {
    build(
        vec![
            AccountMeta::new(*admin, true),
            AccountMeta::new(k.market, false),
            AccountMeta::new_readonly(*collateral_mint, false),
            AccountMeta::new_readonly(*borrow_mint, false),
            AccountMeta::new_readonly(k.collateral_feed, false),
            AccountMeta::new_readonly(k.borrow_feed, false),
            AccountMeta::new(k.collateral_vault, false),
            AccountMeta::new(k.liquidity_vault, false),
            SYSTEM,
            TOKEN,
        ],
        LendingInstruction::InitMarket { params },
    )
}

pub fn supply(supplier: &Pubkey, k: &MarketKeys, source: &Pubkey, amount: u64) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*supplier, true),
            AccountMeta::new(k.market, false),
            AccountMeta::new(k.liquidity_vault, false),
            AccountMeta::new(*source, false),
            TOKEN,
        ],
        LendingInstruction::Supply { amount },
    )
}

pub fn deposit(owner: &Pubkey, k: &MarketKeys, source: &Pubkey, amount: u64) -> Instruction {
    build(
        vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(k.market, false),
            AccountMeta::new(obligation_address(&k.market, owner), false),
            AccountMeta::new(k.collateral_vault, false),
            AccountMeta::new(*source, false),
            SYSTEM,
            TOKEN,
        ],
        LendingInstruction::Deposit { amount },
    )
}

pub fn borrow(owner: &Pubkey, k: &MarketKeys, destination: &Pubkey, amount: u64) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(k.market, false),
            AccountMeta::new(obligation_address(&k.market, owner), false),
            AccountMeta::new(k.liquidity_vault, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(k.collateral_feed, false),
            AccountMeta::new_readonly(k.borrow_feed, false),
            TOKEN,
        ],
        LendingInstruction::Borrow { amount },
    )
}

/// Repay `amount` of `owner`'s debt (anyone may), from `payer`'s `source`.
pub fn repay(
    payer: &Pubkey,
    owner: &Pubkey,
    k: &MarketKeys,
    source: &Pubkey,
    amount: u64,
) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*payer, true),
            AccountMeta::new(k.market, false),
            AccountMeta::new(obligation_address(&k.market, owner), false),
            AccountMeta::new(k.liquidity_vault, false),
            AccountMeta::new(*source, false),
            TOKEN,
        ],
        LendingInstruction::Repay { amount },
    )
}

pub fn withdraw(owner: &Pubkey, k: &MarketKeys, destination: &Pubkey, amount: u64) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(k.market, false),
            AccountMeta::new(obligation_address(&k.market, owner), false),
            AccountMeta::new(k.collateral_vault, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(k.collateral_feed, false),
            AccountMeta::new_readonly(k.borrow_feed, false),
            TOKEN,
        ],
        LendingInstruction::Withdraw { amount },
    )
}

pub fn liquidate(
    liquidator: &Pubkey,
    owner: &Pubkey,
    k: &MarketKeys,
    repay_source: &Pubkey,
    collateral_destination: &Pubkey,
    max_repay: u64,
) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*liquidator, true),
            AccountMeta::new(k.market, false),
            AccountMeta::new(obligation_address(&k.market, owner), false),
            AccountMeta::new(k.liquidity_vault, false),
            AccountMeta::new(k.collateral_vault, false),
            AccountMeta::new(*repay_source, false),
            AccountMeta::new(*collateral_destination, false),
            AccountMeta::new_readonly(k.collateral_feed, false),
            AccountMeta::new_readonly(k.borrow_feed, false),
            TOKEN,
        ],
        LendingInstruction::Liquidate { max_repay },
    )
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    todo!("Exercise 5")
}

fn save<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    todo!("Exercise 5")
}

fn now() -> Result<i64, ProgramError> {
    todo!("Exercise 5")
}

fn init(program_id: &Pubkey, accounts: &[AccountInfo], params: MarketParams) -> ProgramResult {
    todo!("Exercise 5")
}

/// The market, brought up to date.
fn load_market(program_id: &Pubkey, market: &AccountInfo) -> Result<Market, ProgramError> {
    todo!("Exercise 5")
}

fn check_vault(
    program_id: &Pubkey,
    market: &AccountInfo,
    vault: &AccountInfo,
    seed: &[u8],
    bump: u8,
) -> ProgramResult {
    todo!("Exercise 5")
}

fn load_obligation(
    program_id: &Pubkey,
    info: &AccountInfo,
    market: &AccountInfo,
) -> Result<Obligation, ProgramError> {
    todo!("Exercise 5")
}

/// (collateral value, debt value) of an obligation, in micro-units, from
/// validated oracle prices.
fn values(
    state: &Market,
    obligation: &Obligation,
    collateral_feed: &AccountInfo,
    borrow_feed: &AccountInfo,
) -> Result<(u128, u128), ProgramError> {
    todo!("Exercise 5")
}

/// Both prices, scaled to `VALUE_DECIMALS`, after checking they're the
/// market's feeds and fresh and precise enough.
fn prices(
    state: &Market,
    collateral_feed: &AccountInfo,
    borrow_feed: &AccountInfo,
) -> Result<(u64, u64), ProgramError> {
    todo!("Exercise 5")
}

fn market_seeds(state: &Market) -> [Vec<u8>; 4] {
    todo!("Exercise 5")
}

fn supply_liquidity(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    todo!("Exercise 5")
}

fn deposit_collateral(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    todo!("Exercise 5")
}

fn borrow_tokens(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    todo!("Exercise 5")
}

/// Reduce `position`'s debt by up to `amount`; returns what was actually repaid.
fn apply_repayment(
    state: &mut Market,
    position: &mut Obligation,
    amount: u64,
) -> Result<u64, ProgramError> {
    todo!("Exercise 5")
}

fn repay_debt(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    todo!("Exercise 5")
}

fn withdraw_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    todo!("Exercise 5")
}

fn liquidate_position(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    max_repay: u64,
) -> ProgramResult {
    todo!("Exercise 5")
}
