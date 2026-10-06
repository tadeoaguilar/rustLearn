//! Exercise 5: the notes program -- PDAs, and creating, updating and closing
//! accounts.
//!
//! A Program Derived Address (PDA) is an address computed from seeds and a
//! program id that is guaranteed to have no private key. Only that program
//! can "sign" for it, with `invoke_signed` and the seeds. That's how a
//! program creates accounts at predictable addresses and controls them.

use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::ex04_notes_state::{MAX_BODY_LEN, MAX_TITLE_LEN, Note, NoteInstruction};

/// The program's own errors. On the wire they're `ProgramError::Custom(n)`,
/// `n` being the discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum NoteError {
    #[error("title is empty or longer than 32 bytes")]
    BadTitle = 0,
    #[error("body is longer than 280 bytes")]
    BodyTooLong = 1,
    #[error("the note account isn't the PDA for this author and id")]
    WrongNoteAddress = 2,
    #[error("only the author may change a note")]
    NotTheAuthor = 3,
}

impl From<NoteError> for ProgramError {
    fn from(e: NoteError) -> Self {
        todo!("Exercise 5")
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    todo!("Exercise 5")
}

fn create(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    title: String,
    body: String,
) -> ProgramResult {
    todo!("Exercise 5")
}

/// Load a note for an author-only operation, with every check that makes
/// that safe.
fn load_for_author(
    program_id: &Pubkey,
    author: &AccountInfo,
    note: &AccountInfo,
) -> Result<Note, ProgramError> {
    todo!("Exercise 5")
}

fn update(program_id: &Pubkey, accounts: &[AccountInfo], body: String) -> ProgramResult {
    todo!("Exercise 5")
}

fn delete(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    todo!("Exercise 5")
}
