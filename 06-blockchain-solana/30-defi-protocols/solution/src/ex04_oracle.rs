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
        ProgramError::Custom(e as u32)
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
    let iter = &mut accounts.iter();
    let authority = next_account_info(iter)?;
    let feed = next_account_info(iter)?;
    require_signer(authority)?;
    match OracleInstruction::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?
    {
        OracleInstruction::Create { id, expo } => {
            let system_program = next_account_info(iter)?;
            let id_bytes = id.to_le_bytes();
            let bump = require_pda(
                feed,
                &[b"feed", authority.key.as_ref(), &id_bytes],
                program_id,
            )?;
            create_pda_account(
                authority,
                feed,
                system_program,
                PriceFeed::SPACE,
                program_id,
                &[b"feed", authority.key.as_ref(), &id_bytes, &[bump]],
            )?;
            let state = PriceFeed {
                authority: *authority.key,
                id,
                price: 0,
                conf: 0,
                expo,
                publish_time: 0,
                bump,
            };
            save(&state, feed)
        }
        OracleInstruction::Publish { price, conf } => {
            if feed.owner != program_id {
                return Err(ProgramError::IncorrectProgramId);
            }
            let mut state = PriceFeed::deserialize(&mut &feed.try_borrow_data()?[..])
                .map_err(|_| ProgramError::InvalidAccountData)?;
            if state.authority != *authority.key {
                return Err(OracleError::NotTheAuthority.into());
            }
            state.price = price;
            state.conf = conf;
            state.publish_time = Clock::get()?.unix_timestamp;
            save(&state, feed)
        }
    }
}

fn save(state: &PriceFeed, info: &AccountInfo) -> ProgramResult {
    let bytes = borsh::to_vec(state).map_err(|_| ProgramError::InvalidAccountData)?;
    info.try_borrow_mut_data()?[..bytes.len()].copy_from_slice(&bytes);
    Ok(())
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
        let shift = self.expo + decimals;
        if shift >= 0 {
            self.price
                .saturating_mul(10u64.saturating_pow(shift as u32))
        } else {
            self.price / 10u64.saturating_pow((-shift) as u32)
        }
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
    if *feed.owner != ID {
        return Err(OracleError::NotAFeed.into());
    }
    let state = PriceFeed::deserialize(&mut &feed.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    validate(&state, now, max_age, max_conf_bps)
}

/// The checks of `read_price`, on decoded state.
pub fn validate(
    state: &PriceFeed,
    now: i64,
    max_age: i64,
    max_conf_bps: u64,
) -> Result<Price, ProgramError> {
    if state.price <= 0 {
        return Err(OracleError::NonPositivePrice.into());
    }
    if now.saturating_sub(state.publish_time) > max_age {
        return Err(OracleError::StalePrice.into());
    }
    let price = state.price as u64;
    if state.conf as u128 * 10_000 > price as u128 * max_conf_bps as u128 {
        return Err(OracleError::ConfidenceTooWide.into());
    }
    Ok(Price {
        price,
        expo: state.expo,
    })
}
