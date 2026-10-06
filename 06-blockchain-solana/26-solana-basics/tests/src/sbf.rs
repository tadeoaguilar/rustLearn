//! The on-chain track: the programs built by `./build-sbf.sh`, run in
//! LiteSVM with real signatures and compute metering.

use crate::WHICH;
use crate::sut::{ex03_hello, ex04_notes_state, ex06_vault};
use ex04_notes_state::Note;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_signer::Signer;
use solsim::sbf::Vm;

fn vm() -> Vm {
    let mut vm = Vm::new();
    vm.load(ex03_hello::ID, WHICH, "hello");
    vm.load(ex04_notes_state::ID, WHICH, "notes");
    vm.load(ex06_vault::ID, WHICH, "vault");
    vm
}

#[test]
fn sbf_hello() {
    let mut vm = vm();
    let payer = vm.wallet(SOL);
    let meta = vm
        .process(&[ex03_hello::hello("VM", None)], &[&payer])
        .unwrap();
    assert_eq!(
        meta.return_data,
        Some((ex03_hello::ID, b"Hello, VM!".to_vec()))
    );
    assert!(
        meta.logs.iter().any(|l| l == "Program log: Hello, VM!"),
        "{:?}",
        meta.logs
    );
    println!("hello used {} compute units", meta.compute_units);
}

#[test]
fn sbf_notes_lifecycle() {
    let mut vm = vm();
    let author = vm.wallet(SOL);
    let a = author.pubkey();
    vm.process(
        &[ex04_notes_state::create_note(&a, 1, "gm", "on-chain")],
        &[&author],
    )
    .unwrap();
    let (pda, _) = ex04_notes_state::note_address(&a, 1);
    assert_eq!(Note::read(&vm.data(&pda)).unwrap().body, "on-chain");

    vm.process(
        &[ex04_notes_state::update_note(&a, 1, "edited")],
        &[&author],
    )
    .unwrap();
    assert_eq!(Note::read(&vm.data(&pda)).unwrap().body, "edited");

    let mallory = vm.wallet(SOL);
    let mut forged = ex04_notes_state::update_note(&a, 1, "pwned");
    forged.accounts[0].pubkey = mallory.pubkey();
    let err = vm.process(&[forged], &[&mallory]).unwrap_err();
    assert_eq!(err.custom_code(), Some(ex05_not_the_author()));

    vm.process(&[ex04_notes_state::delete_note(&a, 1)], &[&author])
        .unwrap();
    assert!(vm.account(&pda).is_none());
}

fn ex05_not_the_author() -> u32 {
    crate::sut::ex05_notes::NoteError::NotTheAuthor as u32
}

#[test]
fn sbf_vault() {
    let mut vm = vm();
    let user = vm.wallet(3 * SOL);
    let u = user.pubkey();
    let (vault, _) = ex06_vault::vault_address(&u);
    vm.process(&[ex06_vault::deposit(&u, 2 * SOL)], &[&user])
        .unwrap();
    assert_eq!(vm.lamports(&vault), 2 * SOL);
    vm.process(&[ex06_vault::withdraw(&u, SOL)], &[&user])
        .unwrap();
    assert_eq!(vm.lamports(&vault), SOL);

    let thief = vm.wallet(SOL);
    let mut steal = ex06_vault::withdraw(&thief.pubkey(), SOL);
    steal.accounts[1].pubkey = vault;
    assert!(vm.process(&[steal], &[&thief]).is_err());
    assert_eq!(vm.lamports(&vault), SOL);
}
