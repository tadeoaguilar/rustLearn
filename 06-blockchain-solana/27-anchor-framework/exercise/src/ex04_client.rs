//! Exercise 4: a typed client -- what tests (and frontends) are built on.
//!
//! Anchor generates, for every instruction, a `crate::instruction::<Name>`
//! struct holding its arguments (`InstructionData::data()` encodes
//! discriminator + Borsh), and a `crate::accounts::<Name>` struct holding
//! the account addresses (`ToAccountMetas` gives the metas with the right
//! signer/writable flags). A client wraps them so callers only supply what
//! varies, and derives every PDA itself.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solsim::{Sim, TxError};

use crate::errors::LabError;

const SYSTEM: Pubkey = anchor_lang::solana_program::system_program::ID;

pub fn counter_address(authority: &Pubkey) -> Pubkey {
    todo!("Exercise 4")
}

pub fn poll_address(creator: &Pubkey, poll_id: u64) -> Pubkey {
    todo!("Exercise 4")
}

pub fn receipt_address(poll: &Pubkey, voter: &Pubkey) -> Pubkey {
    todo!("Exercise 4")
}

pub fn registry_address() -> Pubkey {
    todo!("Exercise 4")
}

pub fn profile_address(owner: &Pubkey) -> Pubkey {
    todo!("Exercise 4")
}

/// An instruction to this program from Anchor's generated structs.
fn ix(accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
    todo!("Exercise 4")
}

pub fn initialize_counter_ix(authority: &Pubkey) -> Instruction {
    todo!("Exercise 4")
}

pub fn increment_ix(authority: &Pubkey, by: u64) -> Instruction {
    todo!("Exercise 4")
}

pub fn create_poll_ix(
    creator: &Pubkey,
    poll_id: u64,
    question: &str,
    options: &[&str],
    ends_at: i64,
) -> Instruction {
    todo!("Exercise 4")
}

pub fn vote_ix(voter: &Pubkey, poll: &Pubkey, option: u8) -> Instruction {
    todo!("Exercise 4")
}

pub fn create_profile_ix(owner: &Pubkey, handle: &str) -> Instruction {
    todo!("Exercise 4")
}

pub fn verify_profile_ix(admin: &Pubkey, profile_owner: &Pubkey) -> Instruction {
    todo!("Exercise 4")
}

/// Decode an account of type `T`, if it exists, is owned by this program and
/// has `T`'s discriminator.
pub fn fetch<T: AccountDeserialize>(sim: &Sim, key: &Pubkey) -> Option<T> {
    todo!("Exercise 4")
}

/// Every `LabError`, in declaration order (= code - 6000).
pub const ALL_ERRORS: [LabError; 12] = [
    LabError::CounterUnderflow,
    LabError::CounterOverflow,
    LabError::BadQuestion,
    LabError::BadOptions,
    LabError::EndsInThePast,
    LabError::NoSuchOption,
    LabError::PollEnded,
    LabError::PollRunning,
    LabError::BadHandle,
    LabError::NotAdmin,
    LabError::AlreadyVerified,
    LabError::BioTooLong,
];

/// Which `LabError` a failed transaction returned, if it was one.
pub fn lab_error(err: &TxError) -> Option<LabError> {
    todo!("Exercise 4")
}
