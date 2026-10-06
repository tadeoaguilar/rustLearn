//! The program's errors. Anchor numbers them from 6000 in declaration order
//! (`CounterUnderflow` is `Custom(6000)`), after its own 100-5999 range
//! (2000s for constraint failures, 3000s for account errors).

use anchor_lang::prelude::*;

#[error_code]
pub enum LabError {
    #[msg("the counter can't go below zero")]
    CounterUnderflow,
    #[msg("the counter overflowed")]
    CounterOverflow,
    #[msg("a question is 1 to 100 bytes")]
    BadQuestion,
    #[msg("a poll has 2 to 4 options of 1 to 32 bytes")]
    BadOptions,
    #[msg("the poll must end in the future")]
    EndsInThePast,
    #[msg("there is no such option")]
    NoSuchOption,
    #[msg("the poll has ended")]
    PollEnded,
    #[msg("the poll is still running")]
    PollRunning,
    #[msg("a handle is 3 to 16 lowercase letters, digits or underscores")]
    BadHandle,
    #[msg("only the registry admin may do that")]
    NotAdmin,
    #[msg("the profile is already verified")]
    AlreadyVerified,
    #[msg("a bio is at most 200 bytes")]
    BioTooLong,
}
