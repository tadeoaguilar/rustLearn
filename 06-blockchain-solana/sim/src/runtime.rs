//! Executing instructions: account frames, the account rules, CPIs.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use solana_program::account_info::{AccountInfo, MAX_PERMITTED_DATA_INCREASE};
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;

use crate::{Account, Entrypoint, Sim, SimError, TxError, TxMeta};

/// Solana's limit: the transaction's instruction is height 1, so at most 4
/// nested CPIs.
const MAX_STACK_HEIGHT: usize = 5;

/// Per-thread execution context. Tests run on many threads at once, each
/// with its own `Sim`; the syscall stubs are global, so they find the
/// current transaction here.
pub(crate) struct Ctx {
    pub programs: HashMap<Pubkey, Entrypoint>,
    pub clock: Clock,
    pub rent: Rent,
    pub stack: Vec<Frame>,
    pub return_data: Option<(Pubkey, Vec<u8>)>,
    pub logs: Vec<String>,
    /// The first error inside a CPI. A failed CPI fails the whole
    /// transaction even if the caller ignores the `Err` it gets back.
    pub cpi_error: Option<SimError>,
}

/// One program invocation on the call stack.
pub(crate) struct Frame {
    pub program_id: Pubkey,
    /// Account state at the start of the invocation, refreshed around each
    /// CPI it makes: changes are checked against this.
    pub pre: HashMap<Pubkey, Account>,
    pub writable: HashSet<Pubkey>,
}

thread_local! {
    static CTX: RefCell<Option<Ctx>> = const { RefCell::new(None) };
}

/// Run `f` on the current transaction's context, if there is one.
pub(crate) fn with_ctx<R>(f: impl FnOnce(&mut Ctx) -> R) -> Option<R> {
    CTX.with(|c| c.borrow_mut().as_mut().map(f))
}

/// The privileges an account has in one invocation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Meta {
    pub key: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

pub(crate) fn process_transaction(
    sim: &mut Sim,
    instructions: &[Instruction],
    signers: &[Pubkey],
) -> Result<TxMeta, TxError> {
    let signer_set: HashSet<Pubkey> = signers.iter().copied().collect();
    // Like a compiled message: an account is writable in every instruction if
    // any instruction (or being the fee payer) makes it writable.
    let mut writable: HashSet<Pubkey> = signers.first().copied().into_iter().collect();
    let mut keys = HashSet::new();
    for (index, ix) in instructions.iter().enumerate() {
        keys.insert(ix.program_id);
        for meta in &ix.accounts {
            keys.insert(meta.pubkey);
            if meta.is_writable {
                writable.insert(meta.pubkey);
            }
            if meta.is_signer && !signer_set.contains(&meta.pubkey) {
                return Err(TxError {
                    index,
                    error: SimError::MissingSignature(meta.pubkey),
                    logs: vec![],
                });
            }
        }
    }
    keys.extend(signers.iter().copied());
    let before = sim.snapshot(&keys);
    let mut working = before.clone();

    let previous = CTX.with(|c| {
        c.borrow_mut().replace(Ctx {
            programs: sim.programs.clone(),
            clock: sim.clock.clone(),
            rent: sim.rent.clone(),
            stack: Vec::new(),
            return_data: None,
            logs: Vec::new(),
            cpi_error: None,
        })
    });
    assert!(
        previous.is_none(),
        "solsim: a transaction is already running on this thread"
    );
    let finish = || CTX.with(|c| c.borrow_mut().take()).expect("context");

    for (index, ix) in instructions.iter().enumerate() {
        let metas: Vec<Meta> = ix
            .accounts
            .iter()
            .map(|m| Meta {
                key: m.pubkey,
                is_signer: signer_set.contains(&m.pubkey),
                is_writable: writable.contains(&m.pubkey),
            })
            .collect();
        let state = metas
            .iter()
            .map(|m| (m.key, working[&m.key].clone()))
            .collect();
        let result = execute(ix.program_id, &metas, &ix.data, &state);
        let cpi_error = with_ctx(|c| c.cpi_error.take()).flatten();
        match (result, cpi_error) {
            (Ok(post), None) => working.extend(post),
            (result, cpi_error) => {
                let ctx = finish();
                let error = cpi_error.unwrap_or_else(|| result.expect_err("an error"));
                return Err(TxError {
                    index,
                    error,
                    logs: ctx.logs,
                });
            }
        }
    }
    let ctx = finish();

    // Rent: an account that changed may not be left with lamports below the
    // rent-exempt minimum. (0 lamports is fine: the account is deleted.)
    for (key, account) in &working {
        if account != &before[key]
            && account.lamports > 0
            && account.lamports < ctx.rent.minimum_balance(account.data.len())
        {
            return Err(TxError {
                index: instructions.len().saturating_sub(1),
                error: SimError::InsufficientFundsForRent(*key),
                logs: ctx.logs,
            });
        }
    }
    for (key, account) in working {
        if account.lamports == 0 {
            sim.accounts.remove(&key);
        } else {
            sim.accounts.insert(key, account);
        }
    }
    Ok(TxMeta {
        logs: ctx.logs,
        return_data: ctx.return_data,
    })
}

/// Run one program invocation over `state` (the accounts it's given) and
/// return the accounts afterwards, after checking the account rules.
pub(crate) fn execute(
    program_id: Pubkey,
    metas: &[Meta],
    data: &[u8],
    state: &HashMap<Pubkey, Account>,
) -> Result<HashMap<Pubkey, Account>, SimError> {
    let (entry, height) = with_ctx(|c| (c.programs.get(&program_id).copied(), c.stack.len() + 1))
        .expect("solsim: no transaction running");
    let entry = entry.ok_or(SimError::ProgramNotFound(program_id))?;
    let writable: HashSet<Pubkey> = metas
        .iter()
        .filter(|m| m.is_writable)
        .map(|m| m.key)
        .collect();
    with_ctx(|c| {
        c.logs
            .push(format!("Program {program_id} invoke [{height}]"));
        c.stack.push(Frame {
            program_id,
            pre: state.clone(),
            writable: writable.clone(),
        });
    });

    // Lay the accounts out in memory the way the real runtime does, so that
    // `AccountInfo::resize` (realloc) and `original_data_len` work.
    let mut order: Vec<Pubkey> = Vec::new();
    for m in metas {
        if !order.contains(&m.key) {
            order.push(m.key);
        }
    }
    let mut slots: Vec<Slot> = order.iter().map(|k| Slot::new(*k, &state[k])).collect();
    let result = {
        let mut infos_by_key: HashMap<Pubkey, AccountInfo> = HashMap::new();
        for (slot, key) in slots.iter_mut().zip(&order) {
            let meta = metas.iter().find(|m| m.key == *key).expect("meta");
            infos_by_key.insert(*key, slot.account_info(meta.is_signer, meta.is_writable));
        }
        // Duplicates share one AccountInfo (and its RefCells), as on-chain.
        let infos: Vec<AccountInfo> = metas.iter().map(|m| infos_by_key[&m.key].clone()).collect();
        entry(&program_id, &infos, data)
    };
    let post: HashMap<Pubkey, Account> =
        slots.iter().map(|s| (s.key.key, s.to_account())).collect();
    let frame = with_ctx(|c| c.stack.pop()).flatten().expect("frame");

    let checked = result.map_err(SimError::Program).and_then(|()| {
        verify(&program_id, &frame.pre, &post, &frame.writable)?;
        let sum =
            |m: &HashMap<Pubkey, Account>| m.values().map(|a| a.lamports as u128).sum::<u128>();
        if sum(state) != sum(&post) {
            return Err(SimError::UnbalancedInstruction);
        }
        Ok(())
    });
    with_ctx(|c| match &checked {
        Ok(()) => c.logs.push(format!("Program {program_id} success")),
        Err(e) => c.logs.push(format!("Program {program_id} failed: {e:?}")),
    });
    checked.map(|()| post)
}

/// The account rules, checked for every account a program was given.
fn verify(
    program_id: &Pubkey,
    pre: &HashMap<Pubkey, Account>,
    post: &HashMap<Pubkey, Account>,
    writable: &HashSet<Pubkey>,
) -> Result<(), SimError> {
    for (key, after) in post {
        let before = &pre[key];
        if before == after {
            continue;
        }
        let key = *key;
        let is_writable = writable.contains(&key);
        let owned = before.owner == *program_id;
        if before.executable {
            return Err(SimError::ExecutableModified(key));
        }
        if before.owner != after.owner
            && !(owned && is_writable && after.data.iter().all(|b| *b == 0))
        {
            return Err(SimError::ModifiedProgramId(key));
        }
        if after.lamports < before.lamports && !(owned && is_writable) {
            return Err(SimError::ExternalAccountLamportSpend(key));
        }
        if after.lamports != before.lamports && !is_writable {
            return Err(SimError::ReadonlyLamportChange(key));
        }
        if after.data != before.data {
            if !is_writable {
                return Err(SimError::ReadonlyDataModified(key));
            }
            if !owned {
                return Err(SimError::ExternalAccountDataModified(key));
            }
        }
    }
    Ok(())
}

/// The CPI syscall: called (through the syscall stubs) by `invoke_signed`.
pub(crate) fn invoke_signed(
    ix: &Instruction,
    infos: &[AccountInfo],
    signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
    match try_invoke(ix, infos, signers_seeds) {
        Ok(()) => Ok(()),
        Err(error) => {
            let returned = match &error {
                SimError::Program(e) => e.clone(),
                SimError::MissingAccount(_) => ProgramError::NotEnoughAccountKeys,
                SimError::PrivilegeEscalation(_) => ProgramError::MissingRequiredSignature,
                SimError::InvalidSeeds => ProgramError::InvalidSeeds,
                _ => ProgramError::InvalidArgument,
            };
            with_ctx(|c| {
                c.cpi_error.get_or_insert(error);
            });
            Err(returned)
        }
    }
}

fn try_invoke(
    ix: &Instruction,
    infos: &[AccountInfo],
    signers_seeds: &[&[&[u8]]],
) -> Result<(), SimError> {
    let (caller, height, on_stack) = with_ctx(|c| {
        let caller = c.stack.last().expect("CPI outside a program").program_id;
        let on_stack = c.stack.iter().any(|f| f.program_id == ix.program_id);
        (caller, c.stack.len(), on_stack)
    })
    .expect("solsim: CPI outside Sim::process");
    if height >= MAX_STACK_HEIGHT {
        return Err(SimError::CallDepthExceeded);
    }
    if on_stack && ix.program_id != caller {
        return Err(SimError::ReentrancyNotAllowed(ix.program_id));
    }
    let mut pda_signers = HashSet::new();
    for seeds in signers_seeds {
        let pda =
            Pubkey::create_program_address(seeds, &caller).map_err(|_| SimError::InvalidSeeds)?;
        pda_signers.insert(pda);
    }

    // Privileges can be passed on, never escalated.
    let mut metas: Vec<Meta> = Vec::new();
    let mut current: HashMap<Pubkey, Account> = HashMap::new();
    for m in &ix.accounts {
        let info = infos
            .iter()
            .find(|i| *i.key == m.pubkey)
            .ok_or(SimError::MissingAccount(m.pubkey))?;
        if m.is_signer && !(info.is_signer || pda_signers.contains(&m.pubkey)) {
            return Err(SimError::PrivilegeEscalation(m.pubkey));
        }
        if m.is_writable && !info.is_writable {
            return Err(SimError::PrivilegeEscalation(m.pubkey));
        }
        // Merge duplicate metas: an account is a signer/writable if any says so.
        match metas.iter_mut().find(|x| x.key == m.pubkey) {
            Some(x) => {
                x.is_signer |= m.is_signer;
                x.is_writable |= m.is_writable;
            }
            None => metas.push(Meta {
                key: m.pubkey,
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            }),
        }
        current.insert(m.pubkey, read_info(info)?);
    }
    // Callee metas keep their own (de-duplicated) order of first appearance,
    // but the AccountInfo list the callee gets mirrors ix.accounts.
    let callee_metas: Vec<Meta> = ix
        .accounts
        .iter()
        .map(|m| *metas.iter().find(|x| x.key == m.pubkey).expect("merged"))
        .collect();

    // Check the caller's changes so far, then make them the new baseline.
    with_ctx(|c| {
        let frame = c.stack.last_mut().expect("frame");
        for key in current.keys() {
            if !frame.pre.contains_key(key) {
                return Err(SimError::MissingAccount(*key));
            }
        }
        let pre: HashMap<Pubkey, Account> =
            current.keys().map(|k| (*k, frame.pre[k].clone())).collect();
        verify(&caller, &pre, &current, &frame.writable)?;
        frame.pre.extend(current.clone());
        c.return_data = None;
        Ok(())
    })
    .expect("ctx")?;

    let post = execute(ix.program_id, &callee_metas, &ix.data, &current)?;

    // Write the callee's changes back into the caller's AccountInfos.
    for (key, account) in &post {
        for info in infos.iter().filter(|i| i.key == key) {
            write_info(info, account)?;
        }
    }
    with_ctx(|c| c.stack.last_mut().expect("frame").pre.extend(post));
    Ok(())
}

fn read_info(info: &AccountInfo) -> Result<Account, SimError> {
    Ok(Account {
        lamports: info.try_lamports().map_err(SimError::Program)?,
        data: info.try_borrow_data().map_err(SimError::Program)?.to_vec(),
        owner: *info.owner,
        executable: info.executable,
    })
}

fn write_info(info: &AccountInfo, account: &Account) -> Result<(), SimError> {
    let program = SimError::Program;
    **info.try_borrow_mut_lamports().map_err(program)? = account.lamports;
    if info.data_len() != account.data.len() {
        info.resize(account.data.len()).map_err(|_| {
            SimError::Unsupported(format!("{}: account grew by more than 10 KiB", info.key))
        })?;
    }
    info.try_borrow_mut_data()
        .map_err(program)?
        .copy_from_slice(&account.data);
    if *info.owner != account.owner {
        info.assign(&account.owner);
    }
    Ok(())
}

/// `[u32 padding][u32 original data length][key]`: the runtime's input
/// layout puts the original data length just before the key, where
/// `AccountInfo::original_data_len` reads it.
#[repr(C)]
struct KeySlot {
    _padding: u32,
    original_data_len: u32,
    key: Pubkey,
}

/// One account's memory for the duration of an invocation.
struct Slot {
    key: Box<KeySlot>,
    owner: Box<Pubkey>,
    lamports: Box<u64>,
    /// `[u64 data length][data][MAX_PERMITTED_DATA_INCREASE spare bytes]`:
    /// `AccountInfo::resize` writes the new length just before the data.
    buf: Vec<u8>,
    executable: bool,
}

impl Slot {
    fn new(key: Pubkey, account: &Account) -> Self {
        let len = account.data.len();
        let mut buf = vec![0u8; 8 + len + MAX_PERMITTED_DATA_INCREASE];
        buf[..8].copy_from_slice(&(len as u64).to_le_bytes());
        buf[8..8 + len].copy_from_slice(&account.data);
        Slot {
            key: Box::new(KeySlot {
                _padding: 0,
                original_data_len: len as u32,
                key,
            }),
            owner: Box::new(account.owner),
            lamports: Box::new(account.lamports),
            buf,
            executable: account.executable,
        }
    }

    fn data_len(&self) -> usize {
        u64::from_le_bytes(self.buf[..8].try_into().expect("8 bytes")) as usize
    }

    fn account_info(&mut self, is_signer: bool, is_writable: bool) -> AccountInfo<'_> {
        let len = self.data_len();
        // SAFETY: the pointers all point into this slot's heap allocations,
        // which outlive the returned AccountInfo (it borrows `self`). The data
        // slice starts 8 bytes into `buf`, which has room for the length
        // header before it and MAX_PERMITTED_DATA_INCREASE bytes after it --
        // exactly what `AccountInfo::resize` relies on. The owner is written
        // through its shared reference only by `AccountInfo::assign`, the
        // same way the on-chain runtime's serialized input is used.
        unsafe {
            let data = std::slice::from_raw_parts_mut(self.buf.as_mut_ptr().add(8), len);
            let lamports = &mut *(&mut *self.lamports as *mut u64);
            let key = &*(&self.key.key as *const Pubkey);
            let owner = &*(&*self.owner as *const Pubkey);
            AccountInfo::new(
                key,
                is_signer,
                is_writable,
                lamports,
                data,
                owner,
                self.executable,
            )
        }
    }

    fn to_account(&self) -> Account {
        let len = self.data_len();
        Account {
            lamports: *self.lamports,
            data: self.buf[8..8 + len].to_vec(),
            owner: *self.owner,
            executable: self.executable,
        }
    }
}
