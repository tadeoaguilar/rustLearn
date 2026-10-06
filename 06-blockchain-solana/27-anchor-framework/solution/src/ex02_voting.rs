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
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        space = 8 + Poll::INIT_SPACE,
        seeds = [b"poll", creator.key().as_ref(), &poll_id.to_le_bytes()],
        bump,
    )]
    pub poll: Account<'info, Poll>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        mut,
        seeds = [b"poll", poll.creator.as_ref(), &poll.poll_id.to_le_bytes()],
        bump = poll.bump,
    )]
    pub poll: Account<'info, Poll>,
    #[account(
        init,
        payer = voter,
        space = 8 + VoteReceipt::INIT_SPACE,
        seeds = [b"vote", poll.key().as_ref(), voter.key().as_ref()],
        bump,
    )]
    pub receipt: Account<'info, VoteReceipt>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePoll<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut, has_one = creator, close = creator)]
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
        require!(
            !question.is_empty() && question.len() <= MAX_QUESTION_LEN,
            LabError::BadQuestion
        );
        require!(
            (2..=MAX_OPTIONS).contains(&options.len())
                && options
                    .iter()
                    .all(|o| !o.is_empty() && o.len() <= MAX_OPTION_LEN),
            LabError::BadOptions
        );
        require!(
            ends_at > Clock::get()?.unix_timestamp,
            LabError::EndsInThePast
        );

        let poll = &mut self.poll;
        poll.creator = self.creator.key();
        poll.poll_id = poll_id;
        poll.question = question;
        poll.votes = vec![0; options.len()];
        poll.options = options;
        poll.ends_at = ends_at;
        poll.bump = bumps.poll;
        Ok(())
    }
}

impl CastVote<'_> {
    pub fn vote(&mut self, option: u8) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp < self.poll.ends_at,
            LabError::PollEnded
        );
        let tally = self
            .poll
            .votes
            .get_mut(option as usize)
            .ok_or(LabError::NoSuchOption)?;
        *tally = tally.checked_add(1).ok_or(LabError::CounterOverflow)?;

        self.receipt.poll = self.poll.key();
        self.receipt.voter = self.voter.key();
        self.receipt.option = option;
        Ok(())
    }
}

impl ClosePoll<'_> {
    pub fn close(&mut self) -> Result<()> {
        require!(
            Clock::get()?.unix_timestamp >= self.poll.ends_at,
            LabError::PollRunning
        );
        Ok(())
    }
}

/// The winning option(s): every option with the most votes (client side).
pub fn winners(poll: &Poll) -> Vec<&str> {
    let max = poll.votes.iter().copied().max().unwrap_or(0);
    poll.options
        .iter()
        .zip(&poll.votes)
        .filter(|(_, v)| **v == max)
        .map(|(o, _)| o.as_str())
        .collect()
}
