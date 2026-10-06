//! Bonus: growing (and shrinking) an account with `realloc`.
//!
//! A profile is created just big enough for its handle. `set_bio` resizes it
//! to fit the new bio: Anchor tops up the rent deposit from the owner when
//! it grows and refunds the difference when it shrinks.

use anchor_lang::prelude::*;

use crate::errors::LabError;
use crate::ex03_registry::{MAX_BIO_LEN, Profile};

#[derive(Accounts)]
#[instruction(bio: String)]
pub struct SetBio<'info> {
    // TODO Bonus: constraints
    pub owner: Signer<'info>,
    // TODO Bonus: constraints
    pub profile: Account<'info, Profile>,
    pub system_program: Program<'info, System>,
}

impl SetBio<'_> {
    pub fn set_bio(&mut self, bio: String) -> Result<()> {
        todo!("Bonus")
    }
}
