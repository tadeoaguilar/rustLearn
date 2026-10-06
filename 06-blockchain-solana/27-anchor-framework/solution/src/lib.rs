//! Module 27 -- Anchor Framework. Reference solution.
//!
//! One Anchor program, `anchor_lab`, with three features and a client:
//!
//! | File               | Exercise |
//! |--------------------|----------|
//! | `ex01_counter.rs`  | 1  a counter: `init` with PDA seeds, `has_one`, custom errors, `close` |
//! | `ex02_voting.rs`   | 2  polls and votes: `#[instruction]` args in seeds, the Clock, one vote per voter |
//! | `ex03_registry.rs` | 3  account validation: constraints, admin checks, type confusion |
//! | `ex04_client.rs`   | 4  a typed client for tests: builders, fetching accounts, error codes |
//! | `bonus_realloc.rs` | bonus: growing an account with `realloc` |
//! | `errors.rs`        | the program's error codes (provided) |
//!
//! Anchor generates the entrypoint, instruction dispatch (an 8-byte
//! discriminator per instruction), account (de)serialization with an 8-byte
//! discriminator per account type, and all the checks the `#[account(...)]`
//! constraints describe. The handlers only hold the business logic.

use anchor_lang::prelude::*;

pub mod bonus_realloc;
pub mod errors;
pub mod ex01_counter;
pub mod ex02_voting;
pub mod ex03_registry;
#[cfg(not(target_os = "solana"))]
pub mod ex04_client;

// `#[program]` needs the account structs (and the client modules Anchor
// generates next to them) at the crate root.
pub use bonus_realloc::*;
pub use ex01_counter::*;
pub use ex02_voting::*;
pub use ex03_registry::*;

declare_id!("DB6xL653fyXQQa9CA3M6bJt8wMr6G4E3VEztz7j4FC59");

#[program]
pub mod anchor_lab {
    use super::*;

    // Each instruction checks its accounts struct, then calls the handler
    // method on it (in the exercise files).

    // ---- Exercise 1: counter
    pub fn initialize_counter(ctx: Context<InitializeCounter>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }
    pub fn increment(ctx: Context<UpdateCounter>, by: u64) -> Result<()> {
        ctx.accounts.increment(by)
    }
    pub fn decrement(ctx: Context<UpdateCounter>, by: u64) -> Result<()> {
        ctx.accounts.decrement(by)
    }
    pub fn reset(ctx: Context<UpdateCounter>) -> Result<()> {
        ctx.accounts.reset()
    }
    pub fn close_counter(ctx: Context<CloseCounter>) -> Result<()> {
        ctx.accounts.close()
    }

    // ---- Exercise 2: voting
    pub fn create_poll(
        ctx: Context<CreatePoll>,
        poll_id: u64,
        question: String,
        options: Vec<String>,
        ends_at: i64,
    ) -> Result<()> {
        ctx.accounts
            .create(poll_id, question, options, ends_at, &ctx.bumps)
    }
    pub fn vote(ctx: Context<CastVote>, option: u8) -> Result<()> {
        ctx.accounts.vote(option)
    }
    pub fn close_poll(ctx: Context<ClosePoll>) -> Result<()> {
        ctx.accounts.close()
    }

    // ---- Exercise 3: registry
    pub fn initialize_registry(ctx: Context<InitializeRegistry>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }
    pub fn create_profile(ctx: Context<CreateProfile>, handle: String) -> Result<()> {
        ctx.accounts.create(handle, &ctx.bumps)
    }
    pub fn verify_profile(ctx: Context<VerifyProfile>) -> Result<()> {
        ctx.accounts.verify()
    }
    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        ctx.accounts.transfer(new_admin)
    }
    pub fn delete_profile(ctx: Context<DeleteProfile>) -> Result<()> {
        ctx.accounts.delete()
    }

    // ---- Bonus: realloc
    pub fn set_bio(ctx: Context<SetBio>, bio: String) -> Result<()> {
        ctx.accounts.set_bio(bio)
    }
}
