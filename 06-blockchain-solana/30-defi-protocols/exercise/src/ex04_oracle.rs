//! Exercise 4: price oracles.
//!
//! Programs can't fetch prices from the internet; an *oracle* program
//! (Pyth, Switchboard) writes them into accounts that other programs read.
//! A price comes with a confidence interval and a publish time, and a
//! careful consumer refuses prices that are stale or too uncertain -- a
//! lending market that liquidates on a five-minute-old price during a crash
//! is giving money away.
//!
//! This is a minimal Pyth-like oracle: a feed account `["feed", authority, id]`
//! that its authority (the publisher) updates, and `read_price` for consumers.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

use crate::util::{create_pda_account, require_pda, require_signer};

solana_program::declare_id!("8WNBACJC1QRGgsQNtsVbn5YoGQbjbzrkbQsYtQuyQTkX");

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum OracleError {
    #[error("only the feed's authority publishes")]
    NotTheAuthority = 0,
    #[error("the price is too old")]
    StalePrice = 1,
    #[error("the confidence interval is too wide")]
    ConfidenceTooWide = 2,
    #[error("the price must be positive")]
    NonPositivePrice = 3,
    #[error("not a feed of the oracle program")]
    NotAFeed = 4,
}

impl From<OracleError> for ProgramError {
    fn from(e: OracleError) -> Self {
        todo!("Exercise 4")
    }
}

/// `price * 10^expo` is the value of one whole token in the quote currency;
/// the true price is within `price ± conf` with high probability.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct PriceFeed {
    pub authority: Pubkey,
    pub id: u64,
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
    pub bump: u8,
}

impl PriceFeed {
    pub const SPACE: usize = 32 + 8 + 8 + 8 + 4 + 8 + 1;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum OracleInstruction {
    /// `[authority (s, w), feed (w), system program]`
    Create { id: u64, expo: i32 },
    /// `[authority (s), feed (w)]` -- publish time is the current clock.
    Publish { price: i64, conf: u64 },
}

// ------------------------------------------------------------------ client

pub fn feed_address(authority: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"feed", authority.as_ref(), &id.to_le_bytes()], &ID).0
}

pub fn create(authority: &Pubkey, id: u64, expo: i32) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(*authority, true),
            AccountMeta::new(feed_address(authority, id), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        data: borsh::to_vec(&OracleInstruction::Create { id, expo }).expect("serialize"),
    }
}

pub fn publish(authority: &Pubkey, id: u64, price: i64, conf: u64) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(feed_address(authority, id), false),
        ],
        data: borsh::to_vec(&OracleInstruction::Publish { price, conf }).expect("serialize"),
    }
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    todo!("Exercise 4")
}

fn save(state: &PriceFeed, info: &AccountInfo) -> ProgramResult {
    todo!("Exercise 4")
}

// ---------------------------------------------------------------- consumer

/// A validated price: the value of one whole token is `price * 10^expo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Price {
    pub price: u64,
    pub expo: i32,
}

impl Price {
    /// The price scaled to `decimals` decimal places (e.g. micro-dollars for 6).
    pub fn scaled(&self, decimals: i32) -> u64 {
        todo!("Exercise 4")
    }
}

/// Read and validate a price from a feed account: owned by the oracle
/// program (anyone can create an account with the same layout!), positive,
/// published within `max_age` seconds of `now`, and with a confidence
/// interval at most `max_conf_bps` of the price.
pub fn read_price(
    feed: &AccountInfo,
    now: i64,
    max_age: i64,
    max_conf_bps: u64,
) -> Result<Price, ProgramError> {
    todo!("Exercise 4")
}

/// The checks of `read_price`, on decoded state.
pub fn validate(
    state: &PriceFeed,
    now: i64,
    max_age: i64,
    max_conf_bps: u64,
) -> Result<Price, ProgramError> {
    todo!("Exercise 4")
}
