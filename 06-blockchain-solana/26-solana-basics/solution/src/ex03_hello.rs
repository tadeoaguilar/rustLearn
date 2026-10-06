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
    Instruction {
        program_id: ID,
        accounts: counter
            .map(|c| AccountMeta::new(c, false))
            .into_iter()
            .collect(),
        data: name.as_bytes().to_vec(),
    }
}

/// The program's entrypoint.
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let name = std::str::from_utf8(data).map_err(|_| ProgramError::InvalidInstructionData)?;
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return Err(ProgramError::InvalidInstructionData);
    }
    let greeting = format!("Hello, {name}!");
    msg!("{}", greeting);
    set_return_data(greeting.as_bytes());

    if let Some(counter) = accounts.first() {
        // Never trust an account just because it was passed in: anyone can
        // pass any account. Check that it's ours and that we may write it.
        if counter.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        if !counter.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }
        let mut data = counter.try_borrow_mut_data()?;
        let bytes: &mut [u8; COUNTER_LEN] = data
            .get_mut(..COUNTER_LEN)
            .and_then(|s| s.try_into().ok())
            .ok_or(ProgramError::AccountDataTooSmall)?;
        let count = u64::from_le_bytes(*bytes)
            .checked_add(1)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        *bytes = count.to_le_bytes();
        msg!("greetings so far: {}", count);
    }
    Ok(())
}

/// Read a counter account's value (client side).
pub fn read_counter(data: &[u8]) -> Option<u64> {
    let bytes: [u8; COUNTER_LEN] = data.get(..COUNTER_LEN)?.try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}
