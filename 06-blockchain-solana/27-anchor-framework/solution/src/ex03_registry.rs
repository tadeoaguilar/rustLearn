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
        32 + (4 + handle_len) + 1 + (4 + bio_len) + 1
    }
}

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Registry::INIT_SPACE,
        seeds = [b"registry"],
        bump,
    )]
    pub registry: Account<'info, Registry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(handle: String)]
pub struct CreateProfile<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"registry"], bump = registry.bump)]
    pub registry: Account<'info, Registry>,
    #[account(
        init,
        payer = owner,
        space = 8 + Profile::space(handle.len(), 0),
        seeds = [b"profile", owner.key().as_ref()],
        bump,
    )]
    pub profile: Account<'info, Profile>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyProfile<'info> {
    pub admin: Signer<'info>,
    #[account(seeds = [b"registry"], bump = registry.bump, has_one = admin @ LabError::NotAdmin)]
    pub registry: Account<'info, Registry>,
    #[account(
        mut,
        seeds = [b"profile", profile.owner.as_ref()],
        bump = profile.bump,
        constraint = !profile.verified @ LabError::AlreadyVerified,
    )]
    pub profile: Account<'info, Profile>,
}

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"registry"], bump = registry.bump, has_one = admin @ LabError::NotAdmin)]
    pub registry: Account<'info, Registry>,
}

#[derive(Accounts)]
pub struct DeleteProfile<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"registry"], bump = registry.bump)]
    pub registry: Account<'info, Registry>,
    #[account(
        mut,
        has_one = owner,
        close = owner,
        seeds = [b"profile", owner.key().as_ref()],
        bump = profile.bump,
    )]
    pub profile: Account<'info, Profile>,
}

/// 3-16 characters of `a-z`, `0-9` and `_`.
pub fn valid_handle(handle: &str) -> bool {
    (3..=MAX_HANDLE_LEN).contains(&handle.len())
        && handle
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

impl InitializeRegistry<'_> {
    pub fn initialize(&mut self, bumps: &InitializeRegistryBumps) -> Result<()> {
        self.registry.admin = self.admin.key();
        self.registry.profiles = 0;
        self.registry.bump = bumps.registry;
        Ok(())
    }
}

impl CreateProfile<'_> {
    pub fn create(&mut self, handle: String, bumps: &CreateProfileBumps) -> Result<()> {
        require!(valid_handle(&handle), LabError::BadHandle);
        let profile = &mut self.profile;
        profile.owner = self.owner.key();
        profile.handle = handle;
        profile.verified = false;
        profile.bio = String::new();
        profile.bump = bumps.profile;
        self.registry.profiles = self
            .registry
            .profiles
            .checked_add(1)
            .ok_or(LabError::CounterOverflow)?;
        Ok(())
    }
}

impl VerifyProfile<'_> {
    pub fn verify(&mut self) -> Result<()> {
        self.profile.verified = true;
        Ok(())
    }
}

impl TransferAdmin<'_> {
    pub fn transfer(&mut self, new_admin: Pubkey) -> Result<()> {
        self.registry.admin = new_admin;
        Ok(())
    }
}

impl DeleteProfile<'_> {
    pub fn delete(&mut self) -> Result<()> {
        self.registry.profiles = self.registry.profiles.saturating_sub(1);
        Ok(())
    }
}
