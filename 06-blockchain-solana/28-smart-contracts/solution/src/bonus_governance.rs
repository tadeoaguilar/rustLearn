//! Bonus: the multisig governs itself.
//!
//! Changing the threshold is an instruction that only the *vault* can sign.
//! Nobody holds the vault's key, so the only way to run it is to propose it
//! and execute it like any other instruction: the multisig program CPIs into
//! itself, signed by the vault. (Solana forbids reentrancy, but a program
//! invoking itself directly is allowed.) The owners' rules change only by
//! the owners' rules.

use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::ex01_multisig::{
    ID, MultisigInstruction, load_multisig, validate, vault_address, write,
};

/// The instruction to *propose*: set `multisig`'s threshold.
pub fn set_threshold(multisig: &Pubkey, threshold: u8) -> Instruction {
    Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new_readonly(vault_address(multisig), true),
            AccountMeta::new(*multisig, false),
        ],
        data: borsh::to_vec(&MultisigInstruction::SetThreshold { threshold }).expect("serialize"),
    }
}

/// `SetThreshold`: the vault must have signed -- i.e. this is an executed proposal.
pub(crate) fn set_threshold_via_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    threshold: u8,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let vault = next_account_info(iter)?;
    let multisig_info = next_account_info(iter)?;
    let mut multisig = load_multisig(program_id, multisig_info)?;
    let expected_vault = Pubkey::create_program_address(
        &[b"vault", multisig_info.key.as_ref(), &[multisig.vault_bump]],
        program_id,
    )?;
    if *vault.key != expected_vault || !vault.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    validate(&multisig.owners, threshold)?;
    multisig.threshold = threshold;
    write(&multisig, multisig_info)
}
