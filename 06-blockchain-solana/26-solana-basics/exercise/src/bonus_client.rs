//! Bonus: reading program accounts, like an indexer or a dApp's frontend.
//!
//! On a real cluster, `getProgramAccounts` with a `memcmp` filter returns
//! the accounts a program owns whose bytes match at an offset. The author is
//! the first field of a note, so "notes by this author" is a memcmp at
//! offset 0. `Sim::program_accounts` returns them all; the filter is ours.

use solana_program::pubkey::Pubkey;
use solsim::Sim;

use crate::ex04_notes_state::{ID, Note};

/// The accounts owned by `program_id` whose data has `bytes` at `offset`.
pub fn memcmp_filter(sim: &Sim, program_id: &Pubkey, offset: usize, bytes: &[u8]) -> Vec<Pubkey> {
    todo!("Bonus")
}

/// Every note by `author`, sorted by id.
pub fn notes_by_author(sim: &Sim, author: &Pubkey) -> Vec<Note> {
    todo!("Bonus")
}

/// The next unused note id for `author` (one more than the largest).
pub fn next_note_id(sim: &Sim, author: &Pubkey) -> u64 {
    todo!("Bonus")
}
