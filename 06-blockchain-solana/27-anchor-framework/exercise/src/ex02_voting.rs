//! Exercise 2: polls and votes.
//!
//! A poll is a PDA `["poll", creator, poll_id]`. A vote creates a receipt PDA
//! `["vote", poll, voter]`: because `init` fails when the account already
//! exists, a second vote by the same voter fails without any extra code --
//! the receipt's *existence* is the record.

use anchor_lang::prelude::*;

use crate::errors::LabError;

pub const MAX_QUESTION_LEN: usize = 100;
pub const MAX_OPTIONS: usize = 4;
pub const MAX_OPTION_LEN: usize = 32;

#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub creator: Pubkey,
    pub poll_id: u64,
    #[max_len(100)]
    pub question: String,
    #[max_len(4, 32)]
    pub options: Vec<String>,
    #[max_len(4)]
    pub votes: Vec<u64>,
    /// Unix timestamp; voting is open while `now < ends_at`.
    pub ends_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct VoteReceipt {
    pub poll: Pubkey,
    pub voter: Pubkey,
    pub option: u8,
}

#[derive(Accounts)]
// Instruction arguments the constraints need, in order, from the first.
#[instruction(poll_id: u64)]
pub struct CreatePoll<'info> {
    // TODO Exercise 2: constraints
    pub creator: Signer<'info>,
    // TODO Exercise 2: constraints
    pub poll: Account<'info, Poll>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    // TODO Exercise 2: constraints
    pub voter: Signer<'info>,
    // TODO Exercise 2: constraints
    pub poll: Account<'info, Poll>,
    // TODO Exercise 2: constraints
    pub receipt: Account<'info, VoteReceipt>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePoll<'info> {
    // TODO Exercise 2: constraints
    pub creator: Signer<'info>,
    // TODO Exercise 2: constraints
    pub poll: Account<'info, Poll>,
}

impl CreatePoll<'_> {
    pub fn create(
        &mut self,
        poll_id: u64,
        question: String,
        options: Vec<String>,
        ends_at: i64,
        bumps: &CreatePollBumps,
    ) -> Result<()> {
        todo!("Exercise 2")
    }
}

impl CastVote<'_> {
    pub fn vote(&mut self, option: u8) -> Result<()> {
        todo!("Exercise 2")
    }
}

impl ClosePoll<'_> {
    pub fn close(&mut self) -> Result<()> {
        todo!("Exercise 2")
    }
}

/// The winning option(s): every option with the most votes (client side).
pub fn winners(poll: &Poll) -> Vec<&str> {
    todo!("Exercise 2")
}
