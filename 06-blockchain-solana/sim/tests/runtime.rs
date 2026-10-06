//! The runtime rules solsim enforces, each checked with a tiny program.

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::{Sysvar, clock::Clock};
use solana_system_interface::instruction as system_instruction;
use solsim::{Account, Sim, SimError, token};

const SOL: u64 = 1_000_000_000;

/// A program whose behaviour is picked by the first data byte.
fn tricks(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    match data[0] {
        // write a byte into account 0
        0 => {
            accounts[0].try_borrow_mut_data()?[0] = 42;
            Ok(())
        }
        // move 1 lamport from account 0 to account 1 directly
        1 => {
            **accounts[0].try_borrow_mut_lamports()? -= 1;
            **accounts[1].try_borrow_mut_lamports()? += 1;
            Ok(())
        }
        // mint a lamport out of thin air
        2 => {
            **accounts[0].try_borrow_mut_lamports()? += 1;
            Ok(())
        }
        // create a PDA [b"pda"] funded by account 0 (accounts: payer, pda, system)
        3 => {
            let (pda, bump) = Pubkey::find_program_address(&[b"pda"], program_id);
            let ix = system_instruction::create_account(accounts[0].key, &pda, SOL, 8, program_id);
            invoke_signed(&ix, accounts, &[&[b"pda", &[bump]]])?;
            // and write to it right away: it's ours now
            accounts[1].try_borrow_mut_data()?[0] = 7;
            Ok(())
        }
        // sign a transfer from account 0 with seeds [b"vault", data[1]]
        4 => {
            let ix = system_instruction::transfer(accounts[0].key, accounts[1].key, 1);
            invoke_signed(&ix, accounts, &[&[b"vault", &[data[1]]]])
        }
        // CPI back into ourselves through another program would be reentrancy;
        // invoking the System program with a writable flag we don't have is escalation
        5 => {
            let ix = Instruction {
                program_id: solana_sdk_ids::system_program::ID,
                accounts: vec![
                    AccountMeta::new(*accounts[0].key, true),
                    AccountMeta::new(*accounts[1].key, false),
                ],
                data: system_instruction::transfer(accounts[0].key, accounts[1].key, 1).data,
            };
            invoke(&ix, accounts)
        }
        // report the clock in return data
        6 => {
            let clock = Clock::get()?;
            solana_program::program::set_return_data(&clock.unix_timestamp.to_le_bytes());
            Ok(())
        }
        // fail with a custom error
        7 => Err(ProgramError::Custom(77)),
        // grow account 0 by 100 bytes with realloc
        8 => {
            let len = accounts[0].data_len();
            accounts[0].resize(len + 100)?;
            accounts[0].try_borrow_mut_data()?[len + 99] = 1;
            Ok(())
        }
        // close account 0 into account 1
        9 => {
            let lamports = accounts[0].lamports();
            **accounts[0].try_borrow_mut_lamports()? = 0;
            **accounts[1].try_borrow_mut_lamports()? += lamports;
            accounts[0].resize(0)?;
            accounts[0].assign(&solana_sdk_ids::system_program::ID);
            Ok(())
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn setup() -> (Sim, Pubkey, Pubkey) {
    let mut sim = Sim::new();
    let program = Pubkey::new_unique();
    sim.add_program(program, tricks);
    let payer = sim.funded_wallet(10 * SOL);
    (sim, program, payer)
}

fn ix(program: Pubkey, op: u8, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction {
        program_id: program,
        accounts,
        data: vec![op],
    }
}

#[test]
fn system_transfer_moves_lamports() {
    let mut sim = Sim::new();
    let (alice, bob) = (sim.funded_wallet(2 * SOL), Pubkey::new_unique());
    sim.process_ix(system_instruction::transfer(&alice, &bob, SOL), &[alice])
        .unwrap();
    assert_eq!((sim.lamports(&alice), sim.lamports(&bob)), (SOL, SOL));
}

#[test]
fn unsigned_transfer_is_rejected() {
    let mut sim = Sim::new();
    let (alice, bob) = (sim.funded_wallet(2 * SOL), sim.funded_wallet(SOL));
    let err = sim
        .process_ix(system_instruction::transfer(&alice, &bob, SOL), &[bob])
        .unwrap_err();
    assert_eq!(err.error, SimError::MissingSignature(alice));
}

#[test]
fn transactions_are_atomic() {
    let mut sim = Sim::new();
    let (alice, bob) = (sim.funded_wallet(2 * SOL), Pubkey::new_unique());
    let ixs = [
        system_instruction::transfer(&alice, &bob, SOL),
        system_instruction::transfer(&alice, &bob, 10 * SOL), // too much
    ];
    let err = sim.process(&ixs, &[alice]).unwrap_err();
    assert_eq!(err.index, 1);
    assert_eq!(err.custom_code(), Some(1)); // ResultWithNegativeLamports
    assert_eq!(sim.lamports(&alice), 2 * SOL);
    assert_eq!(sim.lamports(&bob), 0);
}

#[test]
fn leaving_an_account_below_rent_exemption_fails() {
    let mut sim = Sim::new();
    let (alice, bob) = (sim.funded_wallet(SOL), Pubkey::new_unique());
    let err = sim
        .process_ix(system_instruction::transfer(&alice, &bob, 1000), &[alice])
        .unwrap_err();
    assert_eq!(err.error, SimError::InsufficientFundsForRent(bob));
    // emptying an account completely is fine: it's deleted
    sim.process_ix(system_instruction::transfer(&alice, &bob, SOL), &[alice])
        .unwrap();
    assert!(sim.account(&alice).is_none());
}

#[test]
fn only_the_owner_may_write_data() {
    let (mut sim, program, _) = setup();
    let foreign = Pubkey::new_unique();
    sim.set_account(
        foreign,
        Account::new_rent_exempt(vec![0; 8], Pubkey::new_unique()),
    );
    let err = sim
        .process_ix(ix(program, 0, vec![AccountMeta::new(foreign, false)]), &[])
        .unwrap_err();
    assert_eq!(err.error, SimError::ExternalAccountDataModified(foreign));

    let mine = Pubkey::new_unique();
    sim.set_account(mine, Account::new_rent_exempt(vec![0; 8], program));
    let err = sim
        .process_ix(
            ix(program, 0, vec![AccountMeta::new_readonly(mine, false)]),
            &[],
        )
        .unwrap_err();
    assert_eq!(err.error, SimError::ReadonlyDataModified(mine));
    let payer = sim.funded_wallet(SOL);
    sim.process_ix(
        ix(program, 0, vec![AccountMeta::new(mine, false)]),
        &[payer],
    )
    .unwrap();
    assert_eq!(sim.data(&mine)[0], 42);
}

#[test]
fn only_the_owner_may_debit_lamports_and_lamports_are_conserved() {
    let (mut sim, program, payer) = setup();
    let to = sim.funded_wallet(SOL);
    let metas = vec![AccountMeta::new(payer, true), AccountMeta::new(to, false)];
    let err = sim.process_ix(ix(program, 1, metas), &[payer]).unwrap_err();
    assert_eq!(err.error, SimError::ExternalAccountLamportSpend(payer));

    let mine = Pubkey::new_unique();
    sim.set_account(mine, Account::new_rent_exempt(vec![], program));
    sim.airdrop(&mine, 10);
    let metas = vec![AccountMeta::new(mine, false), AccountMeta::new(to, false)];
    sim.process_ix(ix(program, 1, metas), &[payer]).unwrap();

    let err = sim
        .process_ix(
            ix(program, 2, vec![AccountMeta::new(mine, false)]),
            &[payer],
        )
        .unwrap_err();
    assert_eq!(err.error, SimError::UnbalancedInstruction);
}

#[test]
fn a_program_creates_and_writes_its_pda_in_one_instruction() {
    let (mut sim, program, payer) = setup();
    let (pda, _) = Pubkey::find_program_address(&[b"pda"], &program);
    let metas = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(pda, false),
        AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
    ];
    let meta = sim.process_ix(ix(program, 3, metas), &[payer]).unwrap();
    let account = sim.account(&pda).unwrap();
    assert_eq!(
        (account.owner, account.data.as_slice(), account.lamports),
        (program, &[7, 0, 0, 0, 0, 0, 0, 0][..], SOL)
    );
    assert_eq!(meta.logs.len(), 4, "{:?}", meta.logs); // invoke [1], invoke [2], success, success
}

#[test]
fn seeds_must_derive_a_pda_of_the_caller() {
    let (mut sim, program, payer) = setup();
    let metas = |vault| {
        vec![
            AccountMeta::new(vault, false),
            AccountMeta::new(payer, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        ]
    };
    let (vault, bump) = Pubkey::find_program_address(&[b"vault"], &program);
    sim.airdrop(&vault, SOL);
    let signed = Instruction {
        program_id: program,
        accounts: metas(vault),
        data: vec![4, bump],
    };
    sim.process_ix(signed, &[payer]).unwrap();

    // The same seeds, but the vault of another program: our program can't sign for it.
    let (foreign, foreign_bump) = Pubkey::find_program_address(&[b"vault"], &Pubkey::new_unique());
    sim.airdrop(&foreign, SOL);
    let forged = Instruction {
        program_id: program,
        accounts: metas(foreign),
        data: vec![4, foreign_bump],
    };
    let err = sim.process_ix(forged, &[payer]).unwrap_err();
    assert!(
        matches!(err.error, SimError::PrivilegeEscalation(k) if k == foreign)
            || err.error == SimError::InvalidSeeds,
        "{err}"
    );
}

#[test]
fn cpi_privileges_cannot_be_escalated() {
    let (mut sim, program, payer) = setup();
    let to = Pubkey::new_unique();
    sim.airdrop(&to, SOL);
    // the caller has `to` read-only but asks the System program for writable
    let metas = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(to, false),
        AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
    ];
    let err = sim.process_ix(ix(program, 5, metas), &[payer]).unwrap_err();
    assert_eq!(err.error, SimError::PrivilegeEscalation(to));
}

#[test]
fn programs_read_the_clock_and_return_data() {
    let (mut sim, program, payer) = setup();
    sim.set_clock(100, 1_234);
    let meta = sim.process_ix(ix(program, 6, vec![]), &[payer]).unwrap();
    assert_eq!(
        meta.return_data,
        Some((program, 1_234i64.to_le_bytes().to_vec()))
    );
}

#[test]
fn custom_errors_come_back() {
    let (mut sim, program, payer) = setup();
    let err = sim
        .process_ix(ix(program, 7, vec![]), &[payer])
        .unwrap_err();
    assert_eq!(err.custom_code(), Some(77));
    assert!(err.is_program_error(&ProgramError::Custom(77)));
}

#[test]
fn realloc_and_closing_accounts() {
    let (mut sim, program, payer) = setup();
    let acct = Pubkey::new_unique();
    sim.set_account(acct, Account::new_rent_exempt(vec![0; 10], program));
    // growing without adding lamports leaves it below rent exemption
    let err = sim
        .process_ix(
            ix(program, 8, vec![AccountMeta::new(acct, false)]),
            &[payer],
        )
        .unwrap_err();
    assert_eq!(err.error, SimError::InsufficientFundsForRent(acct));
    let top_up = system_instruction::transfer(&payer, &acct, SOL);
    sim.process(
        &[top_up, ix(program, 8, vec![AccountMeta::new(acct, false)])],
        &[payer],
    )
    .unwrap();
    assert_eq!(sim.data(&acct).len(), 110);
    assert_eq!(sim.data(&acct)[109], 1);

    let before = sim.lamports(&payer) + sim.lamports(&acct);
    let metas = vec![AccountMeta::new(acct, false), AccountMeta::new(payer, true)];
    sim.process_ix(ix(program, 9, metas), &[payer]).unwrap();
    assert!(
        sim.account(&acct).is_none(),
        "closed accounts are garbage-collected"
    );
    assert_eq!(sim.lamports(&payer), before);
}

#[test]
fn spl_tokens_and_associated_accounts() {
    let mut sim = Sim::new();
    let payer = sim.funded_wallet(10 * SOL);
    let mint = token::create_mint(&mut sim, &payer, &payer, 6);
    let alice = Pubkey::new_unique();
    let ata = token::create_ata(&mut sim, &payer, &alice, &mint);
    token::mint_to(&mut sim, &mint, &payer, &ata, 5_000_000);
    assert_eq!(token::balance(&sim, &ata), 5_000_000);
    assert_eq!(token::mint_state(&sim, &mint).unwrap().supply, 5_000_000);
    let state = token::token_account_state(&sim, &ata).unwrap();
    assert_eq!((state.owner, state.mint), (alice, mint));
    // creating it again fails; idempotently succeeds
    assert!(
        sim.process_ix(
            solsim::ata::create_associated_token_account(&payer, &alice, &mint),
            &[payer]
        )
        .is_err()
    );
    sim.process_ix(
        solsim::ata::create_associated_token_account_idempotent(&payer, &alice, &mint),
        &[payer],
    )
    .unwrap();
}
