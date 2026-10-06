//! `solsim` -- a small Solana runtime that runs programs natively, in-process.
//!
//! Real Solana programs are compiled to SBF bytecode and run by a validator.
//! For learning and testing that's a lot of machinery, so Phase 6 runs the
//! *same program code* compiled for your machine: a program is just its
//! entrypoint function, `fn(&Pubkey, &[AccountInfo], &[u8]) -> ProgramResult`,
//! and `solsim` plays the part of the runtime around it.
//!
//! What it does, as the real runtime does:
//!
//! - keeps accounts (lamports, data, owner, executable) and applies a
//!   transaction's instructions **atomically**: all succeed or nothing changes
//! - hands each program `AccountInfo`s with the right `is_signer` /
//!   `is_writable` flags, and after it returns **enforces the account rules**:
//!   only the owner may change data or debit lamports, read-only accounts
//!   don't change, lamports are conserved, the owner changes only on zeroed
//!   accounts it owned
//! - executes **cross-program invocations** (`invoke`, `invoke_signed`),
//!   checking signer/writable privileges and PDA signatures, refusing
//!   reentrancy and limiting the call depth
//! - provides the **System program** (create/assign/allocate/transfer), the
//!   real **SPL Token program** (the `spl-token` crate's processor) and the
//!   **Associated Token Account** program
//! - garbage-collects accounts left with 0 lamports and rejects transactions
//!   that leave an account below the **rent-exempt** minimum
//! - answers `Clock::get()` / `Rent::get()`, collects `sol_log_data` events
//!   and return data
//!
//! What it doesn't: signatures (a transaction lists the keys that "signed"),
//! fees, compute units, account size beyond 10 KiB per instruction, the
//! loader, and parallel execution. Everything here can be checked again on
//! the real VM; see the phase's GETTING_STARTED.md.
//!
//! ```
//! use solana_program::{pubkey::Pubkey, native_token::LAMPORTS_PER_SOL};
//! use solana_system_interface::instruction as system_instruction;
//! use solsim::Sim;
//!
//! let mut sim = Sim::new();
//! let (alice, bob) = (Pubkey::new_unique(), Pubkey::new_unique());
//! sim.airdrop(&alice, 2 * LAMPORTS_PER_SOL);
//! sim.process(&[system_instruction::transfer(&alice, &bob, LAMPORTS_PER_SOL)], &[alice]).unwrap();
//! assert_eq!(sim.lamports(&bob), LAMPORTS_PER_SOL);
//! ```

pub mod ata;
mod runtime;
#[cfg(feature = "sbf")]
pub mod sbf;
mod stubs;
mod system;
pub mod token;

use std::collections::{HashMap, HashSet};
use std::fmt;

use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_sdk_ids::{bpf_loader_upgradeable, system_program, sysvar};

/// A program's entrypoint. Both plain `fn process(&Pubkey, &[AccountInfo],
/// &[u8])` functions and Anchor's generated `entry` fit this type.
pub type Entrypoint =
    for<'info> fn(&'info Pubkey, &'info [AccountInfo<'info>], &'info [u8]) -> ProgramResult;

/// One account as the runtime stores it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
}

impl Account {
    /// An account with `data`, owned by `owner`, funded to be rent-exempt.
    pub fn new_rent_exempt(data: Vec<u8>, owner: Pubkey) -> Self {
        Account {
            lamports: Rent::default().minimum_balance(data.len()),
            data,
            owner,
            executable: false,
        }
    }
}

impl Default for Account {
    /// What a never-used address looks like: no lamports, no data, owned by
    /// the System program.
    fn default() -> Self {
        Account {
            lamports: 0,
            data: Vec::new(),
            owner: system_program::ID,
            executable: false,
        }
    }
}

/// Why a transaction failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimError {
    /// The program returned this error (Anchor errors are `Custom(6000+)`).
    Program(ProgramError),
    /// An instruction marks this account as a signer, but it didn't sign.
    MissingSignature(Pubkey),
    /// No program is deployed at this address.
    ProgramNotFound(Pubkey),
    /// A CPI references an account the caller didn't pass to it.
    MissingAccount(Pubkey),
    /// A CPI asked for signer or writable privileges the caller doesn't have.
    PrivilegeEscalation(Pubkey),
    /// `invoke_signed` seeds don't derive a valid PDA of the calling program.
    InvalidSeeds,
    /// A program invoked a program that's already on the call stack (other
    /// than invoking itself directly).
    ReentrancyNotAllowed(Pubkey),
    /// Too many nested CPIs.
    CallDepthExceeded,
    /// A program changed the data of an account it doesn't own.
    ExternalAccountDataModified(Pubkey),
    /// A program took lamports from an account it doesn't own.
    ExternalAccountLamportSpend(Pubkey),
    /// Data of an account passed read-only was changed.
    ReadonlyDataModified(Pubkey),
    /// Lamports of an account passed read-only changed.
    ReadonlyLamportChange(Pubkey),
    /// The owner changed without the rules being met (the old owner must do
    /// it, on a writable account whose data is zeroed).
    ModifiedProgramId(Pubkey),
    /// A program account was modified.
    ExecutableModified(Pubkey),
    /// The instruction created or destroyed lamports.
    UnbalancedInstruction,
    /// An account was left with lamports, but fewer than rent exemption
    /// requires for its size.
    InsufficientFundsForRent(Pubkey),
    /// A runtime limit of this simulator (not of Solana) was hit.
    Unsupported(String),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SimError {}

/// A failed transaction: which instruction failed, why, and the logs so far.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxError {
    pub index: usize,
    pub error: SimError,
    pub logs: Vec<String>,
}

impl TxError {
    /// The program's custom error code, if that's why it failed.
    pub fn custom_code(&self) -> Option<u32> {
        match self.error {
            SimError::Program(ProgramError::Custom(code)) => Some(code),
            _ => None,
        }
    }

    /// `true` if the program returned exactly this error.
    pub fn is_program_error(&self, error: &ProgramError) -> bool {
        self.error == SimError::Program(error.clone())
    }
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "instruction {} failed: {:?}", self.index, self.error)?;
        for line in &self.logs {
            writeln!(f, "  {line}")?;
        }
        Ok(())
    }
}

impl std::error::Error for TxError {}

/// What a successful transaction produced.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TxMeta {
    /// "Program X invoke [1]", "Program data: ..." (events), "Program X success".
    pub logs: Vec<String>,
    /// The last `set_return_data` of the transaction, and which program set it.
    pub return_data: Option<(Pubkey, Vec<u8>)>,
}

impl TxMeta {
    /// Decoded `sol_log_data` payloads (Anchor's `emit!`), in order.
    pub fn events(&self) -> Vec<Vec<Vec<u8>>> {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD;
        self.logs
            .iter()
            .filter_map(|l| l.strip_prefix("Program data: "))
            .map(|rest| rest.split(' ').filter_map(|f| b64.decode(f).ok()).collect())
            .collect()
    }
}

/// The in-process runtime: accounts, deployed programs and the clock.
#[derive(Clone)]
pub struct Sim {
    accounts: HashMap<Pubkey, Account>,
    programs: HashMap<Pubkey, Entrypoint>,
    clock: Clock,
    rent: Rent,
}

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

impl Sim {
    /// A fresh runtime with the System and SPL Token programs deployed and the
    /// Clock and Rent sysvar accounts present.
    pub fn new() -> Self {
        stubs::install();
        let mut sim = Sim {
            accounts: HashMap::new(),
            programs: HashMap::new(),
            clock: Clock {
                slot: 1,
                unix_timestamp: 1_700_000_000,
                ..Clock::default()
            },
            rent: Rent::default(),
        };
        sim.add_program(system_program::ID, system::process);
        sim.add_program(spl_token_interface::ID, spl_token_entry);
        sim.add_program(ata::ID, ata::process);
        sim.write_sysvars();
        sim
    }

    /// Deploy `entrypoint` at `program_id`.
    pub fn add_program(&mut self, program_id: Pubkey, entrypoint: Entrypoint) {
        self.programs.insert(program_id, entrypoint);
        self.accounts.insert(
            program_id,
            Account {
                lamports: 1,
                data: Vec::new(),
                owner: bpf_loader_upgradeable::ID,
                executable: true,
            },
        );
    }

    /// Give `key` free lamports (devnet's airdrop, without the faucet).
    pub fn airdrop(&mut self, key: &Pubkey, lamports: u64) {
        self.accounts.entry(*key).or_default().lamports += lamports;
    }

    /// A new address funded with `lamports`: a "wallet" for tests.
    pub fn funded_wallet(&mut self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.airdrop(&key, lamports);
        key
    }

    /// Put an account into the ledger directly (test setup, "genesis").
    pub fn set_account(&mut self, key: Pubkey, account: Account) {
        self.accounts.insert(key, account);
    }

    /// The account at `key`, if it exists (has lamports).
    pub fn account(&self, key: &Pubkey) -> Option<&Account> {
        self.accounts.get(key)
    }

    /// The account's data, or an empty slice.
    pub fn data(&self, key: &Pubkey) -> &[u8] {
        self.accounts.get(key).map_or(&[], |a| &a.data)
    }

    /// The account's balance, 0 if it doesn't exist.
    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.accounts.get(key).map_or(0, |a| a.lamports)
    }

    /// Every account owned by `program_id`, like the `getProgramAccounts` RPC
    /// call, sorted by address.
    pub fn program_accounts(&self, program_id: &Pubkey) -> Vec<(Pubkey, Account)> {
        let mut found: Vec<(Pubkey, Account)> = self
            .accounts
            .iter()
            .filter(|(_, a)| a.owner == *program_id)
            .map(|(k, a)| (*k, a.clone()))
            .collect();
        found.sort_by_key(|(k, _)| *k);
        found
    }

    pub fn rent(&self) -> &Rent {
        &self.rent
    }

    /// The rent-exempt minimum for `data_len` bytes of data.
    pub fn minimum_balance(&self, data_len: usize) -> u64 {
        self.rent.minimum_balance(data_len)
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    /// Move the clock forward (or set it) -- the next transaction sees it.
    pub fn set_clock(&mut self, slot: u64, unix_timestamp: i64) {
        self.clock.slot = slot;
        self.clock.unix_timestamp = unix_timestamp;
        self.write_sysvars();
    }

    /// Advance the wall clock by `seconds` (and the slot by about 2.5/s).
    pub fn advance_time(&mut self, seconds: i64) {
        let slots = (seconds.max(0) as u64).saturating_mul(5) / 2;
        self.set_clock(self.clock.slot + slots, self.clock.unix_timestamp + seconds);
    }

    /// Run a transaction: `instructions` in order, signed by `signers` (the
    /// first one is the fee payer, although fees aren't charged). Atomic: if
    /// any instruction fails, no account changes.
    pub fn process(
        &mut self,
        instructions: &[Instruction],
        signers: &[Pubkey],
    ) -> Result<TxMeta, TxError> {
        runtime::process_transaction(self, instructions, signers)
    }

    /// `process` with a single instruction.
    pub fn process_ix(
        &mut self,
        instruction: Instruction,
        signers: &[Pubkey],
    ) -> Result<TxMeta, TxError> {
        self.process(&[instruction], signers)
    }

    // The sysvar accounts hold the same values `Clock::get()` / `Rent::get()`
    // return, for programs that take them as accounts (the SPL Token
    // program's `InitializeMint` does). Rent's fields are deprecated for
    // direct use, but they are its serialized layout.
    #[allow(deprecated)]
    fn write_sysvars(&mut self) {
        let mut clock = Vec::with_capacity(40);
        clock.extend_from_slice(&self.clock.slot.to_le_bytes());
        clock.extend_from_slice(&self.clock.epoch_start_timestamp.to_le_bytes());
        clock.extend_from_slice(&self.clock.epoch.to_le_bytes());
        clock.extend_from_slice(&self.clock.leader_schedule_epoch.to_le_bytes());
        clock.extend_from_slice(&self.clock.unix_timestamp.to_le_bytes());
        let mut rent = Vec::with_capacity(17);
        rent.extend_from_slice(&self.rent.lamports_per_byte_year.to_le_bytes());
        rent.extend_from_slice(&self.rent.exemption_threshold.to_le_bytes());
        rent.push(self.rent.burn_percent);
        for (key, data) in [(sysvar::clock::ID, clock), (sysvar::rent::ID, rent)] {
            let account = Account {
                lamports: self.rent.minimum_balance(data.len()),
                data,
                owner: sysvar::ID,
                executable: false,
            };
            self.accounts.insert(key, account);
        }
    }

    pub(crate) fn snapshot(&self, keys: &HashSet<Pubkey>) -> HashMap<Pubkey, Account> {
        keys.iter()
            .map(|k| (*k, self.accounts.get(k).cloned().unwrap_or_default()))
            .collect()
    }
}

fn spl_token_entry<'info>(
    program_id: &'info Pubkey,
    accounts: &'info [AccountInfo<'info>],
    data: &'info [u8],
) -> ProgramResult {
    spl_token::processor::Processor::process(program_id, accounts, data)
}
