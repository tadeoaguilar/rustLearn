//! Exercise 3: account validation -- a profile registry.
//!
//! One global `Registry` (PDA `["registry"]`) with an admin and a profile
//! count; one `Profile` per wallet (PDA `["profile", owner]`). The admin
//! verifies profiles. Every check is a constraint:
//!
//! | Constraint | Checks |
//! |---|---|
//! | `Signer<'info>` | the account signed |
//! | `Account<'info, T>` | owned by this program, and its 8-byte discriminator is `T`'s |
//! | `seeds` + `bump` | the address is the PDA |
//! | `has_one = x` | `account.x == x.key()` |
//! | `constraint = expr @ Error` | anything else |

use anchor_lang::prelude::*;

use crate::errors::LabError;

pub const MAX_HANDLE_LEN: usize = 16;
pub const MAX_BIO_LEN: usize = 200;

#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub admin: Pubkey,
    pub profiles: u64,
    pub bump: u8,
}

#[account]
pub struct Profile {
    pub owner: Pubkey,
    pub handle: String,
    pub verified: bool,
    /// Grows with `realloc` (bonus); empty at creation.
    pub bio: String,
    pub bump: u8,
}

impl Profile {
    /// Bytes needed (after the discriminator) for a handle and bio of these lengths.
    pub fn space(handle_len: usize, bio_len: usize) -> usize {
        todo!("Exercise 3")
    }
}

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    // TODO Exercise 3: constraints
    pub admin: Signer<'info>,
    // TODO Exercise 3: constraints
    pub registry: Account<'info, Registry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(handle: String)]
pub struct CreateProfile<'info> {
    // TODO Exercise 3: constraints
    pub owner: Signer<'info>,
    // TODO Exercise 3: constraints
    pub registry: Account<'info, Registry>,
    // TODO Exercise 3: constraints
    pub profile: Account<'info, Profile>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyProfile<'info> {
    pub admin: Signer<'info>,
    // TODO Exercise 3: constraints
    pub registry: Account<'info, Registry>,
    // TODO Exercise 3: constraints
    pub profile: Account<'info, Profile>,
}

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    pub admin: Signer<'info>,
    // TODO Exercise 3: constraints
    pub registry: Account<'info, Registry>,
}

#[derive(Accounts)]
pub struct DeleteProfile<'info> {
    // TODO Exercise 3: constraints
    pub owner: Signer<'info>,
    // TODO Exercise 3: constraints
    pub registry: Account<'info, Registry>,
    // TODO Exercise 3: constraints
    pub profile: Account<'info, Profile>,
}

/// 3-16 characters of `a-z`, `0-9` and `_`.
pub fn valid_handle(handle: &str) -> bool {
    todo!("Exercise 3")
}

impl InitializeRegistry<'_> {
    pub fn initialize(&mut self, bumps: &InitializeRegistryBumps) -> Result<()> {
        todo!("Exercise 3")
    }
}

impl CreateProfile<'_> {
    pub fn create(&mut self, handle: String, bumps: &CreateProfileBumps) -> Result<()> {
        todo!("Exercise 3")
    }
}

impl VerifyProfile<'_> {
    pub fn verify(&mut self) -> Result<()> {
        todo!("Exercise 3")
    }
}

impl TransferAdmin<'_> {
    pub fn transfer(&mut self, new_admin: Pubkey) -> Result<()> {
        todo!("Exercise 3")
    }
}

impl DeleteProfile<'_> {
    pub fn delete(&mut self) -> Result<()> {
        todo!("Exercise 3")
    }
}
