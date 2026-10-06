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
    Pubkey::find_program_address(&[b"counter", authority.as_ref()], &crate::ID).0
}

pub fn poll_address(creator: &Pubkey, poll_id: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"poll", creator.as_ref(), &poll_id.to_le_bytes()],
        &crate::ID,
    )
    .0
}

pub fn receipt_address(poll: &Pubkey, voter: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vote", poll.as_ref(), voter.as_ref()], &crate::ID).0
}

pub fn registry_address() -> Pubkey {
    Pubkey::find_program_address(&[b"registry"], &crate::ID).0
}

pub fn profile_address(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"profile", owner.as_ref()], &crate::ID).0
}

/// An instruction to this program from Anchor's generated structs.
fn ix(accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}

pub fn initialize_counter_ix(authority: &Pubkey) -> Instruction {
    ix(
        crate::accounts::InitializeCounter {
            authority: *authority,
            counter: counter_address(authority),
            system_program: SYSTEM,
        },
        crate::instruction::InitializeCounter {},
    )
}

pub fn increment_ix(authority: &Pubkey, by: u64) -> Instruction {
    ix(
        crate::accounts::UpdateCounter {
            authority: *authority,
            counter: counter_address(authority),
        },
        crate::instruction::Increment { by },
    )
}

pub fn create_poll_ix(
    creator: &Pubkey,
    poll_id: u64,
    question: &str,
    options: &[&str],
    ends_at: i64,
) -> Instruction {
    ix(
        crate::accounts::CreatePoll {
            creator: *creator,
            poll: poll_address(creator, poll_id),
            system_program: SYSTEM,
        },
        crate::instruction::CreatePoll {
            poll_id,
            question: question.to_string(),
            options: options.iter().map(|o| o.to_string()).collect(),
            ends_at,
        },
    )
}

pub fn vote_ix(voter: &Pubkey, poll: &Pubkey, option: u8) -> Instruction {
    ix(
        crate::accounts::CastVote {
            voter: *voter,
            poll: *poll,
            receipt: receipt_address(poll, voter),
            system_program: SYSTEM,
        },
        crate::instruction::Vote { option },
    )
}

pub fn create_profile_ix(owner: &Pubkey, handle: &str) -> Instruction {
    ix(
        crate::accounts::CreateProfile {
            owner: *owner,
            registry: registry_address(),
            profile: profile_address(owner),
            system_program: SYSTEM,
        },
        crate::instruction::CreateProfile {
            handle: handle.to_string(),
        },
    )
}

pub fn verify_profile_ix(admin: &Pubkey, profile_owner: &Pubkey) -> Instruction {
    ix(
        crate::accounts::VerifyProfile {
            admin: *admin,
            registry: registry_address(),
            profile: profile_address(profile_owner),
        },
        crate::instruction::VerifyProfile {},
    )
}

/// Decode an account of type `T`, if it exists, is owned by this program and
/// has `T`'s discriminator.
pub fn fetch<T: AccountDeserialize>(sim: &Sim, key: &Pubkey) -> Option<T> {
    let account = sim.account(key)?;
    if account.owner != crate::ID {
        return None;
    }
    T::try_deserialize(&mut account.data.as_slice()).ok()
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
    let code = err.custom_code()?;
    ALL_ERRORS.into_iter().find(|e| u32::from(*e) == code)
}
