//! Exercise 4: instruction data and account state, with Borsh.
//!
//! Programs and clients agree on byte layouts. Borsh is the usual choice: a
//! deterministic binary format (enums are a one-byte tag, strings a `u32`
//! length then bytes, integers little-endian) that both sides derive.
//!
//! The notes program (Exercise 5) keeps one account per note, at a PDA
//! derived from the author and a note id, so a client can find any note from
//! those two values without an index.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

solana_program::declare_id!("ApEJ8374jKZep9ARnBa49b3J8qWoHgKPF6DhefHbskCF");

pub const MAX_TITLE_LEN: usize = 32;
pub const MAX_BODY_LEN: usize = 280;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum NoteInstruction {
    /// Accounts: `[signer, writable] author`, `[writable] note PDA`, `[] system program`.
    Create {
        id: u64,
        title: String,
        body: String,
    },
    /// Accounts: `[signer] author`, `[writable] note PDA`.
    Update { body: String },
    /// Close the note and return its rent to the author.
    /// Accounts: `[signer, writable] author`, `[writable] note PDA`.
    Delete,
}

impl NoteInstruction {
    /// Decode instruction data; anything malformed (including trailing
    /// bytes) is `InvalidInstructionData`.
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(data).map_err(|_| ProgramError::InvalidInstructionData)
    }

    pub fn pack(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("serializing to a Vec can't fail")
    }
}

/// A note account's contents.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub author: Pubkey,
    pub id: u64,
    pub title: String,
    pub body: String,
    /// The PDA's bump seed, stored so later instructions don't search again.
    pub bump: u8,
}

impl Note {
    /// The account size: enough for the longest title and body. Accounts
    /// don't grow by themselves, so we allocate the maximum up front.
    pub const SPACE: usize = 32 + 8 + (4 + MAX_TITLE_LEN) + (4 + MAX_BODY_LEN) + 1;

    /// Read a note from account data. The data is longer than the encoded
    /// note (it's padded to `SPACE`), so this reads a prefix -- unlike
    /// `try_from_slice`, which rejects trailing bytes.
    pub fn read(data: &[u8]) -> Result<Self, ProgramError> {
        Self::deserialize(&mut &data[..]).map_err(|_| ProgramError::InvalidAccountData)
    }

    /// Write the note at the start of `data`, zero-filling the rest.
    pub fn write(&self, data: &mut [u8]) -> Result<(), ProgramError> {
        let bytes = borsh::to_vec(self).map_err(|_| ProgramError::InvalidAccountData)?;
        if bytes.len() > data.len() {
            return Err(ProgramError::AccountDataTooSmall);
        }
        data[..bytes.len()].copy_from_slice(&bytes);
        data[bytes.len()..].fill(0);
        Ok(())
    }
}

/// The note's address and bump: seeds `["note", author, id as 8 LE bytes]`.
pub fn note_address(author: &Pubkey, id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"note", author.as_ref(), &id.to_le_bytes()], &ID)
}

pub fn create_note(author: &Pubkey, id: u64, title: &str, body: &str) -> Instruction {
    let (note, _) = note_address(author, id);
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(*author, true),
            AccountMeta::new(note, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        data: NoteInstruction::Create {
            id,
            title: title.into(),
            body: body.into(),
        }
        .pack(),
    }
}

pub fn update_note(author: &Pubkey, id: u64, body: &str) -> Instruction {
    let (note, _) = note_address(author, id);
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new_readonly(*author, true),
            AccountMeta::new(note, false),
        ],
        data: NoteInstruction::Update { body: body.into() }.pack(),
    }
}

pub fn delete_note(author: &Pubkey, id: u64) -> Instruction {
    let (note, _) = note_address(author, id);
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(*author, true),
            AccountMeta::new(note, false),
        ],
        data: NoteInstruction::Delete.pack(),
    }
}
