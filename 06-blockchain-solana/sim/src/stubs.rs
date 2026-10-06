//! Syscall stubs: what `invoke`, `Clock::get()` and friends call off-chain.
//!
//! On-chain these are syscalls into the validator. Compiled natively,
//! `solana-program` routes them to a global `SyscallStubs` object, which we
//! replace once with one that finds the running transaction (thread-local
//! context in `runtime`).

use std::sync::Once;

use base64::Engine;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction;
use solana_program::program_stubs::{SyscallStubs, set_syscall_stubs};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;

use crate::runtime::{self, with_ctx};

struct Stubs;

pub(crate) fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        set_syscall_stubs(Box::new(Stubs));
    });
}

impl SyscallStubs for Stubs {
    fn sol_log(&self, message: &str) {
        if with_ctx(|c| c.logs.push(format!("Program log: {message}"))).is_none() {
            println!("{message}");
        }
    }

    fn sol_log_data(&self, fields: &[&[u8]]) {
        let b64 = base64::engine::general_purpose::STANDARD;
        let line = fields
            .iter()
            .map(|f| b64.encode(f))
            .collect::<Vec<_>>()
            .join(" ");
        with_ctx(|c| c.logs.push(format!("Program data: {line}")));
    }

    fn sol_invoke_signed(
        &self,
        instruction: &Instruction,
        account_infos: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        runtime::invoke_signed(instruction, account_infos, signers_seeds)
    }

    fn sol_get_clock_sysvar(&self, var_addr: *mut u8) -> u64 {
        let clock = with_ctx(|c| c.clock.clone()).unwrap_or_default();
        // SAFETY: `Clock::get()` passes a pointer to its own `Clock` value.
        unsafe { std::ptr::write(var_addr as *mut Clock, clock) };
        0
    }

    fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
        let rent = with_ctx(|c| c.rent.clone()).unwrap_or_default();
        // SAFETY: `Rent::get()` passes a pointer to its own `Rent` value.
        unsafe { std::ptr::write(var_addr as *mut Rent, rent) };
        0
    }

    fn sol_set_return_data(&self, data: &[u8]) {
        with_ctx(|c| {
            let program = c.stack.last().map(|f| f.program_id).unwrap_or_default();
            c.return_data = (!data.is_empty()).then(|| (program, data.to_vec()));
        });
    }

    fn sol_get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        with_ctx(|c| c.return_data.clone()).flatten()
    }

    fn sol_get_stack_height(&self) -> u64 {
        with_ctx(|c| c.stack.len() as u64).unwrap_or(0)
    }
}
