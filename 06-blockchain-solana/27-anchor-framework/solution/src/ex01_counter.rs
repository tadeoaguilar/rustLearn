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
    #[account(mut)]
    pub authority: Signer<'info>,
    // `init`: create the account (CPI to the System program), paid by
    // `authority`, sized for the discriminator plus the struct, at the PDA.
    #[account(
        init,
        payer = authority,
        space = 8 + Counter::INIT_SPACE,
        seeds = [b"counter", authority.key().as_ref()],
        bump,
    )]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    pub authority: Signer<'info>,
    // `has_one = authority`: counter.authority == authority.key().
    // The seeds re-derive the address with the stored bump.
    #[account(
        mut,
        has_one = authority,
        seeds = [b"counter", authority.key().as_ref()],
        bump = counter.bump,
    )]
    pub counter: Account<'info, Counter>,
}

#[derive(Accounts)]
pub struct CloseCounter<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    // `close = authority`: after the handler, move all lamports to the
    // authority and wipe the account.
    #[account(
        mut,
        has_one = authority,
        close = authority,
        seeds = [b"counter", authority.key().as_ref()],
        bump = counter.bump,
    )]
    pub counter: Account<'info, Counter>,
}

// The handlers. By the time they run, every constraint above has been
// checked; they only hold the logic.

impl InitializeCounter<'_> {
    pub fn initialize(&mut self, bumps: &InitializeCounterBumps) -> Result<()> {
        self.counter.authority = self.authority.key();
        self.counter.count = 0;
        self.counter.bump = bumps.counter;
        Ok(())
    }
}

impl UpdateCounter<'_> {
    pub fn increment(&mut self, by: u64) -> Result<()> {
        self.counter.count = self
            .counter
            .count
            .checked_add(by)
            .ok_or(LabError::CounterOverflow)?;
        Ok(())
    }

    pub fn decrement(&mut self, by: u64) -> Result<()> {
        self.counter.count = self
            .counter
            .count
            .checked_sub(by)
            .ok_or(LabError::CounterUnderflow)?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.counter.count = 0;
        Ok(())
    }
}

impl CloseCounter<'_> {
    pub fn close(&mut self) -> Result<()> {
        // Everything happens in the `close` constraint.
        Ok(())
    }
}
