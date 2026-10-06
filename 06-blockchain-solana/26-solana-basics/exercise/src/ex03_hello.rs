//! Exercise 3: "Hello, World" -- a whole Solana program.
//!
//! A program is one function: the runtime calls it with the program's own
//! id, the accounts the instruction lists, and the instruction's data bytes.
//! It has no other inputs and no state of its own -- anything it remembers
//! lives in accounts it owns.
//!
//! This one greets a name (the instruction data, UTF-8), logs the greeting,
//! returns it as return data, and -- if given a counter account it owns --
//! counts the greetings there.

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::msg;
use solana_program::program::set_return_data;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

solana_program::declare_id!("3tb79Aa8qFSJeJBi21BDwumsxHJYS5EP5qXdAnKRA6Uu");

/// Names are 1 to 32 bytes.
pub const MAX_NAME_LEN: usize = 32;

/// The counter account's size: one little-endian `u64`.
pub const COUNTER_LEN: usize = 8;

/// The instruction: greet `name`, counting in `counter` if given.
pub fn hello(name: &str, counter: Option<Pubkey>) -> Instruction {
    todo!("Exercise 3")
}

/// The program's entrypoint.
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    todo!("Exercise 3")
}

/// Read a counter account's value (client side).
pub fn read_counter(data: &[u8]) -> Option<u64> {
    todo!("Exercise 3")
}
