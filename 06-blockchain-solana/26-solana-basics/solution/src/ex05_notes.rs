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
        ProgramError::Custom(e as u32)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match NoteInstruction::unpack(data)? {
        NoteInstruction::Create { id, title, body } => {
            create(program_id, accounts, id, title, body)
        }
        NoteInstruction::Update { body } => update(program_id, accounts, body),
        NoteInstruction::Delete => delete(program_id, accounts),
    }
}

fn create(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    title: String,
    body: String,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let author = next_account_info(iter)?;
    let note = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !author.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if title.is_empty() || title.len() > MAX_TITLE_LEN {
        return Err(NoteError::BadTitle.into());
    }
    if body.len() > MAX_BODY_LEN {
        return Err(NoteError::BodyTooLong.into());
    }
    if *system_program.key != solana_system_interface::program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let id_bytes = id.to_le_bytes();
    let (expected, bump) =
        Pubkey::find_program_address(&[b"note", author.key.as_ref(), &id_bytes], program_id);
    if *note.key != expected {
        return Err(NoteError::WrongNoteAddress.into());
    }

    // The System program creates the account; the PDA "signs" through our
    // seeds. If the note already exists, create_account fails with
    // AccountAlreadyInUse -- no separate check needed.
    let lamports = Rent::get()?.minimum_balance(Note::SPACE);
    let create = solana_system_interface::instruction::create_account(
        author.key,
        note.key,
        lamports,
        Note::SPACE as u64,
        program_id,
    );
    invoke_signed(
        &create,
        &[author.clone(), note.clone(), system_program.clone()],
        &[&[b"note", author.key.as_ref(), &id_bytes, &[bump]]],
    )?;

    let state = Note {
        author: *author.key,
        id,
        title,
        body,
        bump,
    };
    state.write(&mut note.try_borrow_mut_data()?)?;
    msg!("note {} created", id);
    Ok(())
}

/// Load a note for an author-only operation, with every check that makes
/// that safe.
fn load_for_author(
    program_id: &Pubkey,
    author: &AccountInfo,
    note: &AccountInfo,
) -> Result<Note, ProgramError> {
    if !author.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // Ours? Otherwise anyone could pass an account they made with the same layout.
    if note.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let state = Note::read(&note.try_borrow_data()?)?;
    if state.author != *author.key {
        return Err(NoteError::NotTheAuthor.into());
    }
    // And the right address for that author and id, using the stored bump.
    let expected = Pubkey::create_program_address(
        &[
            b"note",
            state.author.as_ref(),
            &state.id.to_le_bytes(),
            &[state.bump],
        ],
        program_id,
    )
    .map_err(|_| NoteError::WrongNoteAddress)?;
    if *note.key != expected {
        return Err(NoteError::WrongNoteAddress.into());
    }
    Ok(state)
}

fn update(program_id: &Pubkey, accounts: &[AccountInfo], body: String) -> ProgramResult {
    let iter = &mut accounts.iter();
    let author = next_account_info(iter)?;
    let note = next_account_info(iter)?;
    let mut state = load_for_author(program_id, author, note)?;
    if body.len() > MAX_BODY_LEN {
        return Err(NoteError::BodyTooLong.into());
    }
    state.body = body;
    state.write(&mut note.try_borrow_mut_data()?)
}

fn delete(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let author = next_account_info(iter)?;
    let note = next_account_info(iter)?;
    load_for_author(program_id, author, note)?;

    // Closing: move every lamport out (we own the account, so we may debit
    // it), then wipe it and hand it back to the System program, so the same
    // address could be created again cleanly. With 0 lamports the runtime
    // deletes it at the end of the transaction.
    let lamports = note.lamports();
    **note.try_borrow_mut_lamports()? = 0;
    **author.try_borrow_mut_lamports()? = author
        .lamports()
        .checked_add(lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    note.resize(0)?;
    note.assign(&solana_system_interface::program::ID);
    Ok(())
}
