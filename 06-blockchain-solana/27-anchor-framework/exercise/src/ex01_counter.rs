//! Exercise 1: a counter -- the shape of every Anchor instruction.
//!
//! Each user gets one counter, a PDA `["counter", authority]`. The
//! `#[derive(Accounts)]` struct says which accounts an instruction takes and
//! what must be true of them; Anchor checks it all before the handler runs.

use anchor_lang::prelude::*;

use crate::errors::LabError;

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct InitializeCounter<'info> {
    // TODO Exercise 1: constraints
    pub authority: Signer<'info>,
    // TODO Exercise 1: constraints
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    pub authority: Signer<'info>,
    // TODO Exercise 1: constraints
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct CloseCounter<'info> {
    // TODO Exercise 1: constraints
    pub authority: Signer<'info>,
    // TODO Exercise 1: constraints
    pub counter: Account<'info, Counter>,
}

// The handlers. By the time they run, every constraint above has been
// checked; they only hold the logic.

impl InitializeCounter<'_> {
    pub fn initialize(&mut self, bumps: &InitializeCounterBumps) -> Result<()> {
        todo!("Exercise 1")
    }
}

impl UpdateCounter<'_> {
    pub fn increment(&mut self, by: u64) -> Result<()> {
        todo!("Exercise 1")
    }

    pub fn decrement(&mut self, by: u64) -> Result<()> {
        todo!("Exercise 1")
    }

    pub fn reset(&mut self) -> Result<()> {
        todo!("Exercise 1")
    }
}

impl CloseCounter<'_> {
    pub fn close(&mut self) -> Result<()> {
        todo!("Exercise 1")
    }
}
