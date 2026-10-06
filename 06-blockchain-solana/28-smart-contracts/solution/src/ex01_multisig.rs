//! Exercise 1: a multisig wallet.
//!
//! A group of owners controls a vault (the PDA `["vault", multisig]`, a plain
//! System account holding SOL). Any owner may *propose* an arbitrary
//! instruction; owners *approve*; once `threshold` owners have approved,
//! anyone may *execute* it, and the program performs it as a CPI signed by
//! the vault. That's the whole design of Squads and the SPL multisig: the
//! program doesn't know what it executes -- a transfer, a token transfer, a
//! program upgrade, or (bonus) a change to the multisig itself.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::util::{create_pda_account, require_pda, require_signer};

solana_program::declare_id!("2MHDwk9zn4sByoPKfANSTumtyt9Y6y7R3FqskbqvgV3c");

pub const MAX_OWNERS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[repr(u32)]
pub enum MultisigError {
    #[error("1 to 10 distinct owners")]
    InvalidOwners = 0,
    #[error("the threshold must be between 1 and the number of owners")]
    InvalidThreshold = 1,
    #[error("the signer is not an owner")]
    NotAnOwner = 2,
    #[error("this owner already approved")]
    AlreadyApproved = 3,
    #[error("not enough approvals")]
    NotEnoughApprovals = 4,
    #[error("the proposal was already executed")]
    AlreadyExecuted = 5,
    #[error("the proposal belongs to another multisig")]
    WrongMultisig = 6,
}

impl From<MultisigError> for ProgramError {
    fn from(e: MultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// An `AccountMeta`, storable.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct StoredMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

/// An `Instruction`, storable in an account until it's executed.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct StoredInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<StoredMeta>,
    pub data: Vec<u8>,
}

impl From<&Instruction> for StoredInstruction {
    fn from(ix: &Instruction) -> Self {
        StoredInstruction {
            program_id: ix.program_id,
            accounts: ix
                .accounts
                .iter()
                .map(|m| StoredMeta {
                    pubkey: m.pubkey,
                    is_signer: m.is_signer,
                    is_writable: m.is_writable,
                })
                .collect(),
            data: ix.data.clone(),
        }
    }
}

impl StoredInstruction {
    pub fn to_instruction(&self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: self
                .accounts
                .iter()
                .map(|m| AccountMeta {
                    pubkey: m.pubkey,
                    is_signer: m.is_signer,
                    is_writable: m.is_writable,
                })
                .collect(),
            data: self.data.clone(),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Multisig {
    pub creator: Pubkey,
    pub id: u64,
    pub owners: Vec<Pubkey>,
    pub threshold: u8,
    /// Proposals are numbered 0, 1, 2, ...; this is the next number.
    pub proposal_count: u64,
    pub bump: u8,
    pub vault_bump: u8,
}

impl Multisig {
    /// Room for `MAX_OWNERS` owners, so owners can be changed without realloc.
    pub const SPACE: usize = 32 + 8 + (4 + 32 * MAX_OWNERS) + 1 + 8 + 1 + 1;

    pub fn is_owner(&self, key: &Pubkey) -> bool {
        self.owners.contains(key)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    pub multisig: Pubkey,
    pub index: u64,
    pub proposer: Pubkey,
    pub instruction: StoredInstruction,
    /// Owners who approved. Only those *still* owners count.
    pub approvals: Vec<Pubkey>,
    pub executed: bool,
    pub bump: u8,
}

impl Proposal {
    /// Approvals from current owners.
    pub fn approval_count(&self, multisig: &Multisig) -> usize {
        self.approvals
            .iter()
            .filter(|a| multisig.is_owner(a))
            .count()
    }

    /// The account size: exact, plus room for one approval per possible owner.
    pub fn space(&self) -> usize {
        let without_approvals =
            borsh::to_vec(self).expect("serialize").len() - 32 * self.approvals.len();
        without_approvals + 32 * MAX_OWNERS
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum MultisigInstruction {
    /// `[creator (s, w), multisig (w), system program]`
    Create {
        id: u64,
        owners: Vec<Pubkey>,
        threshold: u8,
    },
    /// `[proposer (s, w), multisig (w), proposal (w), system program]`
    Propose { instruction: StoredInstruction },
    /// `[owner (s), multisig, proposal (w)]`
    Approve,
    /// `[executor (s), multisig (w), proposal (w), vault (w), ...every account
    /// the stored instruction uses, and its program]`
    Execute,
    /// Bonus (`bonus_governance.rs`): `[vault (s), multisig (w)]` -- only
    /// through `Execute`.
    SetThreshold { threshold: u8 },
}

// ------------------------------------------------------------------ client

pub fn multisig_address(creator: &Pubkey, id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"multisig", creator.as_ref(), &id.to_le_bytes()], &ID).0
}

pub fn vault_address(multisig: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", multisig.as_ref()], &ID).0
}

pub fn proposal_address(multisig: &Pubkey, index: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"proposal", multisig.as_ref(), &index.to_le_bytes()], &ID).0
}

fn build(accounts: Vec<AccountMeta>, data: MultisigInstruction) -> Instruction {
    Instruction {
        program_id: ID,
        accounts,
        data: borsh::to_vec(&data).expect("serialize"),
    }
}

pub fn create(creator: &Pubkey, id: u64, owners: &[Pubkey], threshold: u8) -> Instruction {
    build(
        vec![
            AccountMeta::new(*creator, true),
            AccountMeta::new(multisig_address(creator, id), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        MultisigInstruction::Create {
            id,
            owners: owners.to_vec(),
            threshold,
        },
    )
}

pub fn propose(
    proposer: &Pubkey,
    multisig: &Pubkey,
    index: u64,
    instruction: &Instruction,
) -> Instruction {
    build(
        vec![
            AccountMeta::new(*proposer, true),
            AccountMeta::new(*multisig, false),
            AccountMeta::new(proposal_address(multisig, index), false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
        ],
        MultisigInstruction::Propose {
            instruction: instruction.into(),
        },
    )
}

pub fn approve(owner: &Pubkey, multisig: &Pubkey, index: u64) -> Instruction {
    build(
        vec![
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new_readonly(*multisig, false),
            AccountMeta::new(proposal_address(multisig, index), false),
        ],
        MultisigInstruction::Approve,
    )
}

/// `inner` is the proposed instruction: its accounts (and program) must be
/// passed along. The vault can't sign the transaction -- the program signs
/// for it -- so it's passed as a non-signer.
pub fn execute(
    executor: &Pubkey,
    multisig: &Pubkey,
    index: u64,
    inner: &Instruction,
) -> Instruction {
    let vault = vault_address(multisig);
    let mut accounts = vec![
        AccountMeta::new(*executor, true),
        AccountMeta::new(*multisig, false),
        AccountMeta::new(proposal_address(multisig, index), false),
        AccountMeta::new(vault, false),
    ];
    for meta in &inner.accounts {
        let is_signer = meta.is_signer && meta.pubkey != vault;
        accounts.push(AccountMeta {
            pubkey: meta.pubkey,
            is_signer,
            is_writable: meta.is_writable,
        });
    }
    accounts.push(AccountMeta::new_readonly(inner.program_id, false));
    build(accounts, MultisigInstruction::Execute)
}

// ----------------------------------------------------------------- program

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let instruction = MultisigInstruction::try_from_slice(data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;
    match instruction {
        MultisigInstruction::Create {
            id,
            owners,
            threshold,
        } => create_multisig(program_id, accounts, id, owners, threshold),
        MultisigInstruction::Propose { instruction } => {
            propose_instruction(program_id, accounts, instruction)
        }
        MultisigInstruction::Approve => approve_proposal(program_id, accounts),
        MultisigInstruction::Execute => execute_proposal(program_id, accounts),
        MultisigInstruction::SetThreshold { threshold } => {
            crate::bonus_governance::set_threshold_via_vault(program_id, accounts, threshold)
        }
    }
}

pub(crate) fn validate(owners: &[Pubkey], threshold: u8) -> ProgramResult {
    let mut unique = owners.to_vec();
    unique.sort();
    unique.dedup();
    if owners.is_empty() || owners.len() > MAX_OWNERS || unique.len() != owners.len() {
        return Err(MultisigError::InvalidOwners.into());
    }
    if threshold == 0 || threshold as usize > owners.len() {
        return Err(MultisigError::InvalidThreshold.into());
    }
    Ok(())
}

fn create_multisig(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: u64,
    owners: Vec<Pubkey>,
    threshold: u8,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let creator = next_account_info(iter)?;
    let multisig = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    require_signer(creator)?;
    validate(&owners, threshold)?;
    let id_bytes = id.to_le_bytes();
    let bump = require_pda(
        multisig,
        &[b"multisig", creator.key.as_ref(), &id_bytes],
        program_id,
    )?;
    let (_, vault_bump) =
        Pubkey::find_program_address(&[b"vault", multisig.key.as_ref()], program_id);
    create_pda_account(
        creator,
        multisig,
        system_program,
        Multisig::SPACE,
        program_id,
        &[b"multisig", creator.key.as_ref(), &id_bytes, &[bump]],
    )?;
    let state = Multisig {
        creator: *creator.key,
        id,
        owners,
        threshold,
        proposal_count: 0,
        bump,
        vault_bump,
    };
    write(&state, multisig)
}

/// Load the multisig, checking it's one of ours.
pub(crate) fn load_multisig(
    program_id: &Pubkey,
    info: &AccountInfo,
) -> Result<Multisig, ProgramError> {
    if info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    Multisig::deserialize(&mut &info.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)
}

/// Load a proposal, checking it's ours and belongs to `multisig`.
fn load_proposal(
    program_id: &Pubkey,
    info: &AccountInfo,
    multisig: &Pubkey,
) -> Result<Proposal, ProgramError> {
    if info.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let proposal = Proposal::deserialize(&mut &info.try_borrow_data()?[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if proposal.multisig != *multisig {
        return Err(MultisigError::WrongMultisig.into());
    }
    Ok(proposal)
}

pub(crate) fn write<T: BorshSerialize>(state: &T, info: &AccountInfo) -> ProgramResult {
    let bytes = borsh::to_vec(state).map_err(|_| ProgramError::InvalidAccountData)?;
    let mut data = info.try_borrow_mut_data()?;
    if bytes.len() > data.len() {
        return Err(ProgramError::AccountDataTooSmall);
    }
    data[..bytes.len()].copy_from_slice(&bytes);
    data[bytes.len()..].fill(0);
    Ok(())
}

fn propose_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: StoredInstruction,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let proposer = next_account_info(iter)?;
    let multisig_info = next_account_info(iter)?;
    let proposal_info = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;
    require_signer(proposer)?;
    let mut multisig = load_multisig(program_id, multisig_info)?;
    if !multisig.is_owner(proposer.key) {
        return Err(MultisigError::NotAnOwner.into());
    }
    let index = multisig.proposal_count;
    let index_bytes = index.to_le_bytes();
    let bump = require_pda(
        proposal_info,
        &[b"proposal", multisig_info.key.as_ref(), &index_bytes],
        program_id,
    )?;
    let proposal = Proposal {
        multisig: *multisig_info.key,
        index,
        proposer: *proposer.key,
        instruction,
        approvals: vec![*proposer.key],
        executed: false,
        bump,
    };
    create_pda_account(
        proposer,
        proposal_info,
        system_program,
        proposal.space(),
        program_id,
        &[
            b"proposal",
            multisig_info.key.as_ref(),
            &index_bytes,
            &[bump],
        ],
    )?;
    write(&proposal, proposal_info)?;
    multisig.proposal_count += 1;
    write(&multisig, multisig_info)
}

fn approve_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let owner = next_account_info(iter)?;
    let multisig_info = next_account_info(iter)?;
    let proposal_info = next_account_info(iter)?;
    require_signer(owner)?;
    let multisig = load_multisig(program_id, multisig_info)?;
    let mut proposal = load_proposal(program_id, proposal_info, multisig_info.key)?;
    if !multisig.is_owner(owner.key) {
        return Err(MultisigError::NotAnOwner.into());
    }
    if proposal.executed {
        return Err(MultisigError::AlreadyExecuted.into());
    }
    if proposal.approvals.contains(owner.key) {
        return Err(MultisigError::AlreadyApproved.into());
    }
    proposal.approvals.push(*owner.key);
    write(&proposal, proposal_info)
}

fn execute_proposal(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let executor = next_account_info(iter)?;
    let multisig_info = next_account_info(iter)?;
    let proposal_info = next_account_info(iter)?;
    let vault = next_account_info(iter)?;
    let remaining: Vec<AccountInfo> = iter.cloned().collect();
    require_signer(executor)?;
    let multisig = load_multisig(program_id, multisig_info)?;
    let mut proposal = load_proposal(program_id, proposal_info, multisig_info.key)?;
    if proposal.executed {
        return Err(MultisigError::AlreadyExecuted.into());
    }
    if proposal.approval_count(&multisig) < multisig.threshold as usize {
        return Err(MultisigError::NotEnoughApprovals.into());
    }
    let vault_seeds: &[&[u8]] = &[b"vault", multisig_info.key.as_ref(), &[multisig.vault_bump]];
    let expected_vault = Pubkey::create_program_address(vault_seeds, program_id)?;
    if *vault.key != expected_vault {
        return Err(ProgramError::InvalidSeeds);
    }

    // Effects before interactions: mark it executed *before* the CPI, so the
    // stored instruction can never run twice, whatever it does.
    proposal.executed = true;
    write(&proposal, proposal_info)?;

    let mut infos = vec![vault.clone(), multisig_info.clone()];
    infos.extend(remaining);
    invoke_signed(
        &proposal.instruction.to_instruction(),
        &infos,
        &[vault_seeds],
    )
    // Don't write `multisig` after this: the CPI may have changed it (bonus),
    // and our copy is stale.
}
