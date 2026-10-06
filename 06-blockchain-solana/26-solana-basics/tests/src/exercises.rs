use crate::sut::{
    bonus_client, ex01_wallet, ex02_transfers, ex03_hello, ex04_notes_state, ex05_notes, ex06_vault,
};
use ex02_transfers::AccountKind;
use ex04_notes_state::{Note, NoteInstruction};
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_signer::Signer;
use solsim::{Account, Sim, SimError};

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(ex03_hello::ID, ex03_hello::process);
    sim.add_program(ex04_notes_state::ID, ex05_notes::process);
    sim.add_program(ex06_vault::ID, ex06_vault::process);
    sim
}

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_wallets_from_the_same_seed_are_the_same() {
    let a = ex01_wallet::wallet_from_seed(&[1; 32]);
    let b = ex01_wallet::wallet_from_seed(&[1; 32]);
    let c = ex01_wallet::wallet_from_seed(&[2; 32]);
    assert_eq!(a.pubkey(), b.pubkey());
    assert_ne!(a.pubkey(), c.pubkey());
    assert_ne!(
        ex01_wallet::new_wallet().pubkey(),
        ex01_wallet::new_wallet().pubkey()
    );
}

#[test]
fn ex1_keypair_file_round_trip_and_format() {
    let wallet = ex01_wallet::wallet_from_seed(&[9; 32]);
    let json = ex01_wallet::to_keypair_file(&wallet);
    let bytes: Vec<u8> = serde_json::from_str(&json).expect("a JSON array of numbers");
    assert_eq!(bytes.len(), 64);
    assert_eq!(&bytes[..32], &[9; 32], "secret half first");
    assert_eq!(
        &bytes[32..],
        wallet.pubkey().as_ref(),
        "then the public half"
    );
    assert_eq!(
        ex01_wallet::from_keypair_file(&json).unwrap().pubkey(),
        wallet.pubkey()
    );
}

#[test]
fn ex1_bad_keypair_files_are_rejected() {
    use ex01_wallet::WalletError::*;
    assert_eq!(
        ex01_wallet::from_keypair_file("not json").err(),
        Some(BadKeypairFile)
    );
    assert_eq!(
        ex01_wallet::from_keypair_file("[1,2,3]").err(),
        Some(BadKeypairFile)
    );
    let mut bytes = ex01_wallet::wallet_from_seed(&[3; 32]).to_bytes().to_vec();
    bytes[40] ^= 1; // corrupt the public half
    let json = serde_json::to_string(&bytes).unwrap();
    assert_eq!(
        ex01_wallet::from_keypair_file(&json).err(),
        Some(MismatchedKeypair)
    );
}

#[test]
fn ex1_signatures_verify_only_for_the_signer_and_message() {
    let (alice, bob) = (
        ex01_wallet::wallet_from_seed(&[4; 32]),
        ex01_wallet::wallet_from_seed(&[5; 32]),
    );
    let sig = ex01_wallet::sign_message(&alice, b"transfer 1 SOL");
    assert!(ex01_wallet::verify(
        &alice.pubkey(),
        b"transfer 1 SOL",
        &sig
    ));
    assert!(!ex01_wallet::verify(
        &alice.pubkey(),
        b"transfer 9 SOL",
        &sig
    ));
    assert!(!ex01_wallet::verify(&bob.pubkey(), b"transfer 1 SOL", &sig));
}

#[test]
fn ex1_parse_sol_is_exact() {
    let ok = |s: &str| ex01_wallet::parse_sol(s).unwrap();
    assert_eq!(ok("1"), SOL);
    assert_eq!(ok("1.5"), 1_500_000_000);
    assert_eq!(ok("0.000000001"), 1);
    assert_eq!(ok("0.1"), 100_000_000);
    assert_eq!(ok("18446744073.709551615"), u64::MAX);
    for bad in [
        "",
        ".",
        "1.",
        ".5",
        "-1",
        "1.0000000001",
        "abc",
        "1,5",
        "18446744074",
        " 1",
    ] {
        assert!(
            ex01_wallet::parse_sol(bad).is_err(),
            "{bad:?} should be rejected"
        );
    }
}

#[test]
fn ex1_format_sol() {
    assert_eq!(ex01_wallet::format_sol(1_500_000_000), "1.5 SOL");
    assert_eq!(ex01_wallet::format_sol(2 * SOL), "2 SOL");
    assert_eq!(ex01_wallet::format_sol(1), "0.000000001 SOL");
    assert_eq!(ex01_wallet::format_sol(0), "0 SOL");
    assert_eq!(ex01_wallet::format_sol(890_880), "0.00089088 SOL");
}

// ---------------------------------------------------------------- Exercise 2

#[test]
fn ex2_describe_accounts() {
    let sim = sim();
    let wallet = Account {
        lamports: 5,
        ..Account::default()
    };
    assert_eq!(ex02_transfers::describe(&wallet), AccountKind::Wallet);
    assert_eq!(
        ex02_transfers::describe(sim.account(&ex03_hello::ID).unwrap()),
        AccountKind::Program
    );
    assert_eq!(
        ex02_transfers::describe(sim.account(&solana_program::sysvar::clock::ID).unwrap()),
        AccountKind::Sysvar
    );
    let owner = Pubkey::new_unique();
    assert_eq!(
        ex02_transfers::describe(&Account::new_rent_exempt(vec![1], owner)),
        AccountKind::Data { owner }
    );
    // System-owned *with* data isn't a wallet (it can't pay with a transfer).
    let system_data = Account::new_rent_exempt(vec![0; 4], solana_sdk_ids::system_program::ID);
    assert!(matches!(
        ex02_transfers::describe(&system_data),
        AccountKind::Data { .. }
    ));
}

#[test]
fn ex2_transfer_ix_is_a_system_transfer() {
    let (a, b) = (Pubkey::new_unique(), Pubkey::new_unique());
    let ix = ex02_transfers::transfer_ix(&a, &b, 42);
    assert_eq!(ix.program_id, solana_sdk_ids::system_program::ID);
    assert!(ix.accounts[0].is_signer && ix.accounts[0].is_writable && ix.accounts[1].is_writable);
    assert_eq!(ix.data, [2, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0]); // u32 tag 2, u64 amount
}

#[test]
fn ex2_pay_many_pays_everyone_atomically() {
    let mut sim = sim();
    let payer = sim.funded_wallet(3 * SOL);
    let (a, b) = (Pubkey::new_unique(), Pubkey::new_unique());
    ex02_transfers::pay_many(&mut sim, &payer, &[(a, SOL), (b, SOL / 2)]).unwrap();
    assert_eq!(
        (sim.lamports(&a), sim.lamports(&b), sim.lamports(&payer)),
        (SOL, SOL / 2, 3 * SOL / 2)
    );

    let err = ex02_transfers::pay_many(&mut sim, &payer, &[(a, SOL), (b, 5 * SOL)]).unwrap_err();
    assert_eq!(err.index, 1);
    assert_eq!(sim.lamports(&a), SOL, "the first payment was rolled back");
}

#[test]
fn ex2_spendable_keeps_the_wallet_rent_exempt() {
    let mut sim = sim();
    let wallet = sim.funded_wallet(SOL);
    let spendable = ex02_transfers::spendable(&sim, &wallet);
    assert_eq!(spendable, SOL - sim.minimum_balance(0));
    let to = sim.funded_wallet(SOL);
    sim.process_ix(
        ex02_transfers::transfer_ix(&wallet, &to, spendable),
        &[wallet],
    )
    .unwrap();
    assert_eq!(sim.lamports(&wallet), sim.minimum_balance(0));
    assert_eq!(ex02_transfers::spendable(&sim, &Pubkey::new_unique()), 0);
}

#[test]
fn ex2_create_data_account_funds_exactly_rent_exemption() {
    let mut sim = sim();
    let payer = sim.funded_wallet(SOL);
    let (account, owner) = (Pubkey::new_unique(), Pubkey::new_unique());
    ex02_transfers::create_data_account(&mut sim, &payer, &account, 100, &owner).unwrap();
    let created = sim.account(&account).unwrap();
    assert_eq!((created.data.len(), created.owner), (100, owner));
    assert_eq!(created.lamports, sim.minimum_balance(100));
    // the address is taken now
    assert!(ex02_transfers::create_data_account(&mut sim, &payer, &account, 100, &owner).is_err());
}

// ---------------------------------------------------------------- Exercise 3

#[test]
fn ex3_hello_returns_the_greeting() {
    let mut sim = sim();
    let payer = sim.funded_wallet(SOL);
    let ix = ex03_hello::hello("Rust", None);
    assert_eq!(
        (ix.program_id, ix.data.as_slice(), ix.accounts.len()),
        (ex03_hello::ID, &b"Rust"[..], 0)
    );
    let meta = sim.process_ix(ix, &[payer]).unwrap();
    assert_eq!(
        meta.return_data,
        Some((ex03_hello::ID, b"Hello, Rust!".to_vec()))
    );
}

#[test]
fn ex3_names_must_be_1_to_32_bytes_of_utf8() {
    let mut sim = sim();
    let payer = sim.funded_wallet(SOL);
    for data in [vec![], vec![b'x'; 33], vec![0xff, 0xfe]] {
        let ix = Instruction {
            program_id: ex03_hello::ID,
            accounts: vec![],
            data,
        };
        let err = sim.process_ix(ix, &[payer]).unwrap_err();
        assert!(
            err.is_program_error(&ProgramError::InvalidInstructionData),
            "{err}"
        );
    }
    sim.process_ix(ex03_hello::hello(&"x".repeat(32), None), &[payer])
        .unwrap();
}

#[test]
fn ex3_counts_greetings_in_its_own_account() {
    let mut sim = sim();
    let payer = sim.funded_wallet(SOL);
    let counter = Pubkey::new_unique();
    sim.set_account(
        counter,
        Account::new_rent_exempt(vec![0; ex03_hello::COUNTER_LEN], ex03_hello::ID),
    );
    for name in ["a", "b", "c"] {
        sim.process_ix(ex03_hello::hello(name, Some(counter)), &[payer])
            .unwrap();
    }
    assert_eq!(ex03_hello::read_counter(sim.data(&counter)), Some(3));
    assert_eq!(ex03_hello::read_counter(&[1, 2]), None);
}

#[test]
fn ex3_rejects_counters_it_does_not_own_or_cannot_write() {
    let mut sim = sim();
    let payer = sim.funded_wallet(SOL);
    let foreign = Pubkey::new_unique();
    sim.set_account(
        foreign,
        Account::new_rent_exempt(vec![0; 8], Pubkey::new_unique()),
    );
    let err = sim
        .process_ix(ex03_hello::hello("a", Some(foreign)), &[payer])
        .unwrap_err();
    assert!(
        err.is_program_error(&ProgramError::IncorrectProgramId),
        "{err}"
    );

    let counter = Pubkey::new_unique();
    sim.set_account(
        counter,
        Account::new_rent_exempt(vec![0; 8], ex03_hello::ID),
    );
    let mut ix = ex03_hello::hello("a", Some(counter));
    ix.accounts[0].is_writable = false;
    let err = sim.process_ix(ix, &[payer]).unwrap_err();
    assert!(
        err.is_program_error(&ProgramError::InvalidAccountData),
        "{err}"
    );
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_instruction_round_trip_and_layout() {
    let ix = NoteInstruction::Create {
        id: 7,
        title: "t".into(),
        body: "hi".into(),
    };
    let bytes = ix.pack();
    // tag 0, id as u64 LE, then strings as u32 length + bytes
    assert_eq!(
        bytes,
        [
            0, 7, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, b't', 2, 0, 0, 0, b'h', b'i'
        ]
    );
    assert_eq!(NoteInstruction::unpack(&bytes).unwrap(), ix);
    assert_eq!(NoteInstruction::Delete.pack(), [2]);
}

#[test]
fn ex4_malformed_instructions_are_rejected() {
    for bad in [&[][..], &[9], &[2, 0], &[1, 5, 0, 0, 0, b'a']] {
        assert_eq!(
            NoteInstruction::unpack(bad),
            Err(ProgramError::InvalidInstructionData),
            "{bad:?}"
        );
    }
}

#[test]
fn ex4_note_state_reads_back_from_padded_data() {
    let note = Note {
        author: Pubkey::new_unique(),
        id: 3,
        title: "title".into(),
        body: "body".into(),
        bump: 254,
    };
    let mut data = vec![0xAA; Note::SPACE];
    note.write(&mut data).unwrap();
    assert_eq!(Note::read(&data).unwrap(), note);
    let used = borsh::to_vec(&note).unwrap().len();
    assert!(
        data[used..].iter().all(|b| *b == 0),
        "the rest is zero-filled"
    );
    assert_eq!(
        note.write(&mut [0; 10]),
        Err(ProgramError::AccountDataTooSmall)
    );
    assert_eq!(
        Note::read(&[1, 2, 3]),
        Err(ProgramError::InvalidAccountData)
    );
}

#[test]
fn ex4_note_address_and_builders() {
    let author = Pubkey::new_unique();
    let (pda, bump) = ex04_notes_state::note_address(&author, 5);
    let expected = Pubkey::find_program_address(
        &[b"note", author.as_ref(), &5u64.to_le_bytes()],
        &ex04_notes_state::ID,
    );
    assert_eq!((pda, bump), expected);
    assert_ne!(pda, ex04_notes_state::note_address(&author, 6).0);

    let create = ex04_notes_state::create_note(&author, 5, "t", "b");
    assert_eq!(create.program_id, ex04_notes_state::ID);
    let metas: Vec<(Pubkey, bool, bool)> = create
        .accounts
        .iter()
        .map(|m| (m.pubkey, m.is_signer, m.is_writable))
        .collect();
    assert_eq!(
        metas,
        [
            (author, true, true),
            (pda, false, true),
            (solana_sdk_ids::system_program::ID, false, false)
        ]
    );
    assert_eq!(
        NoteInstruction::unpack(&create.data).unwrap(),
        NoteInstruction::Create {
            id: 5,
            title: "t".into(),
            body: "b".into()
        }
    );

    let update = ex04_notes_state::update_note(&author, 5, "new");
    assert_eq!(
        (update.accounts[0].is_signer, update.accounts[1].pubkey),
        (true, pda)
    );
    let delete = ex04_notes_state::delete_note(&author, 5);
    assert!(
        delete.accounts[0].is_writable,
        "the author receives the refund"
    );
    assert_eq!(
        NoteInstruction::unpack(&delete.data).unwrap(),
        NoteInstruction::Delete
    );
}

// ---------------------------------------------------------------- Exercise 5

fn note_error(e: ex05_notes::NoteError) -> ProgramError {
    e.into()
}

#[test]
fn ex5_create_puts_the_note_at_its_pda() {
    let mut sim = sim();
    let author = sim.funded_wallet(SOL);
    sim.process_ix(
        ex04_notes_state::create_note(&author, 1, "gm", "hello"),
        &[author],
    )
    .unwrap();
    let (pda, bump) = ex04_notes_state::note_address(&author, 1);
    let account = sim.account(&pda).expect("note account");
    assert_eq!(account.owner, ex04_notes_state::ID);
    assert_eq!(account.data.len(), Note::SPACE);
    assert_eq!(account.lamports, sim.minimum_balance(Note::SPACE));
    let note = Note::read(&account.data).unwrap();
    assert_eq!(
        note,
        Note {
            author,
            id: 1,
            title: "gm".into(),
            body: "hello".into(),
            bump
        }
    );
}

#[test]
fn ex5_create_validates_its_input() {
    let mut sim = sim();
    let author = sim.funded_wallet(SOL);
    let bad_title = sim
        .process_ix(
            ex04_notes_state::create_note(&author, 1, "", "b"),
            &[author],
        )
        .unwrap_err();
    assert!(bad_title.is_program_error(&note_error(ex05_notes::NoteError::BadTitle)));
    let long_title = "t".repeat(33);
    let err = sim
        .process_ix(
            ex04_notes_state::create_note(&author, 1, &long_title, "b"),
            &[author],
        )
        .unwrap_err();
    assert!(err.is_program_error(&note_error(ex05_notes::NoteError::BadTitle)));
    let long_body = "b".repeat(281);
    let err = sim
        .process_ix(
            ex04_notes_state::create_note(&author, 1, "t", &long_body),
            &[author],
        )
        .unwrap_err();
    assert!(err.is_program_error(&note_error(ex05_notes::NoteError::BodyTooLong)));

    // the note account must be the PDA for (author, id)
    let mut wrong = ex04_notes_state::create_note(&author, 1, "t", "b");
    wrong.accounts[1].pubkey = ex04_notes_state::note_address(&author, 2).0;
    let err = sim.process_ix(wrong, &[author]).unwrap_err();
    assert!(
        err.is_program_error(&note_error(ex05_notes::NoteError::WrongNoteAddress)),
        "{err}"
    );

    // and the author must sign
    let mut unsigned = ex04_notes_state::create_note(&author, 1, "t", "b");
    unsigned.accounts[0].is_signer = false;
    let payer = sim.funded_wallet(SOL);
    let err = sim.process_ix(unsigned, &[payer]).unwrap_err();
    assert!(
        err.is_program_error(&ProgramError::MissingRequiredSignature),
        "{err}"
    );
}

#[test]
fn ex5_a_note_cannot_be_created_twice() {
    let mut sim = sim();
    let author = sim.funded_wallet(SOL);
    sim.process_ix(
        ex04_notes_state::create_note(&author, 1, "a", ""),
        &[author],
    )
    .unwrap();
    let err = sim
        .process_ix(
            ex04_notes_state::create_note(&author, 1, "b", ""),
            &[author],
        )
        .unwrap_err();
    assert_eq!(
        err.custom_code(),
        Some(0),
        "the System program's AccountAlreadyInUse: {err}"
    );
}

#[test]
fn ex5_only_the_author_updates() {
    let mut sim = sim();
    let author = sim.funded_wallet(SOL);
    sim.process_ix(
        ex04_notes_state::create_note(&author, 1, "a", "old"),
        &[author],
    )
    .unwrap();
    sim.process_ix(ex04_notes_state::update_note(&author, 1, "new"), &[author])
        .unwrap();
    let (pda, _) = ex04_notes_state::note_address(&author, 1);
    assert_eq!(Note::read(sim.data(&pda)).unwrap().body, "new");

    let mallory = sim.funded_wallet(SOL);
    let mut forged = ex04_notes_state::update_note(&author, 1, "pwned");
    forged.accounts[0].pubkey = mallory;
    let err = sim.process_ix(forged, &[mallory]).unwrap_err();
    assert!(
        err.is_program_error(&note_error(ex05_notes::NoteError::NotTheAuthor)),
        "{err}"
    );

    let err = sim
        .process_ix(
            ex04_notes_state::update_note(&author, 1, &"x".repeat(281)),
            &[author],
        )
        .unwrap_err();
    assert!(err.is_program_error(&note_error(ex05_notes::NoteError::BodyTooLong)));
}

#[test]
fn ex5_fake_note_accounts_are_rejected() {
    let mut sim = sim();
    let mallory = sim.funded_wallet(SOL);
    // An account with a perfectly valid note layout naming mallory as the
    // author -- but owned by another program.
    let fake = Pubkey::new_unique();
    let note = Note {
        author: mallory,
        id: 1,
        title: "t".into(),
        body: "".into(),
        bump: 255,
    };
    let mut data = vec![0; Note::SPACE];
    note.write(&mut data).unwrap();
    sim.set_account(
        fake,
        Account::new_rent_exempt(data.clone(), Pubkey::new_unique()),
    );
    let mut ix = ex04_notes_state::delete_note(&mallory, 1);
    ix.accounts[1].pubkey = fake;
    let err = sim.process_ix(ix, &[mallory]).unwrap_err();
    assert!(
        err.is_program_error(&ProgramError::IncorrectProgramId),
        "{err}"
    );

    // Owned by the notes program but not at the PDA for (author, id).
    let misplaced = Pubkey::new_unique();
    sim.set_account(
        misplaced,
        Account::new_rent_exempt(data, ex04_notes_state::ID),
    );
    let mut ix = ex04_notes_state::delete_note(&mallory, 1);
    ix.accounts[1].pubkey = misplaced;
    let err = sim.process_ix(ix, &[mallory]).unwrap_err();
    assert!(
        err.is_program_error(&note_error(ex05_notes::NoteError::WrongNoteAddress)),
        "{err}"
    );
}

#[test]
fn ex5_delete_refunds_the_rent_and_frees_the_address() {
    let mut sim = sim();
    let author = sim.funded_wallet(SOL);
    sim.process_ix(
        ex04_notes_state::create_note(&author, 1, "a", ""),
        &[author],
    )
    .unwrap();
    let (pda, _) = ex04_notes_state::note_address(&author, 1);
    let rent = sim.lamports(&pda);
    let before = sim.lamports(&author);
    sim.process_ix(ex04_notes_state::delete_note(&author, 1), &[author])
        .unwrap();
    assert!(sim.account(&pda).is_none());
    assert_eq!(sim.lamports(&author), before + rent);
    // the id can be used again
    sim.process_ix(
        ex04_notes_state::create_note(&author, 1, "again", ""),
        &[author],
    )
    .unwrap();
}

#[test]
fn ex5_unknown_instructions_fail() {
    let mut sim = sim();
    let author = sim.funded_wallet(SOL);
    let ix = Instruction {
        program_id: ex04_notes_state::ID,
        accounts: vec![],
        data: vec![42],
    };
    let err = sim.process_ix(ix, &[author]).unwrap_err();
    assert!(err.is_program_error(&ProgramError::InvalidInstructionData));
}

// ---------------------------------------------------------------- Exercise 6

#[test]
fn ex6_encode_and_decode() {
    assert_eq!(
        ex06_vault::encode(ex06_vault::WITHDRAW, 258),
        [1, 2, 1, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        ex06_vault::decode(&[0, 5, 0, 0, 0, 0, 0, 0, 0]),
        Ok((ex06_vault::DEPOSIT, 5))
    );
    for bad in [
        &[][..],
        &[0],
        &[0, 1, 2, 3, 4, 5, 6, 7],
        &[2, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0; 10],
    ] {
        assert_eq!(
            ex06_vault::decode(bad),
            Err(ProgramError::InvalidInstructionData),
            "{bad:?}"
        );
    }
}

#[test]
fn ex6_deposit_and_withdraw() {
    let mut sim = sim();
    let user = sim.funded_wallet(3 * SOL);
    let (vault, _) = ex06_vault::vault_address(&user);
    assert_eq!(
        vault,
        Pubkey::find_program_address(&[b"vault", user.as_ref()], &ex06_vault::ID).0
    );

    let meta = sim
        .process_ix(ex06_vault::deposit(&user, 2 * SOL), &[user])
        .unwrap();
    assert_eq!((sim.lamports(&vault), sim.lamports(&user)), (2 * SOL, SOL));
    assert!(
        meta.logs
            .iter()
            .any(|l| l.contains("11111111111111111111111111111111 invoke [2]")),
        "a CPI to the System program"
    );

    sim.process_ix(ex06_vault::withdraw(&user, SOL / 2), &[user])
        .unwrap();
    assert_eq!(sim.lamports(&vault), 3 * SOL / 2);
    // emptying the vault completely is allowed (the account disappears)
    sim.process_ix(ex06_vault::withdraw(&user, 3 * SOL / 2), &[user])
        .unwrap();
    assert_eq!(sim.lamports(&user), 3 * SOL);
    assert!(sim.account(&vault).is_none());
}

#[test]
fn ex6_cannot_withdraw_more_than_the_vault_holds() {
    let mut sim = sim();
    let user = sim.funded_wallet(3 * SOL);
    sim.process_ix(ex06_vault::deposit(&user, SOL), &[user])
        .unwrap();
    let err = sim
        .process_ix(ex06_vault::withdraw(&user, 2 * SOL), &[user])
        .unwrap_err();
    assert_eq!(
        err.custom_code(),
        Some(1),
        "ResultWithNegativeLamports: {err}"
    );
}

#[test]
fn ex6_nobody_else_can_empty_a_vault() {
    let mut sim = sim();
    let user = sim.funded_wallet(3 * SOL);
    let (vault, _) = ex06_vault::vault_address(&user);
    sim.process_ix(ex06_vault::deposit(&user, 2 * SOL), &[user])
        .unwrap();
    let thief = sim.funded_wallet(SOL);

    // the thief's own instruction, pointed at the user's vault
    let mut steal = ex06_vault::withdraw(&thief, SOL);
    steal.accounts[1].pubkey = vault;
    let err = sim.process_ix(steal, &[thief]).unwrap_err();
    assert!(err.is_program_error(&ProgramError::InvalidSeeds), "{err}");

    // the user's instruction without the user's signature
    let mut unsigned = ex06_vault::withdraw(&user, SOL);
    unsigned.accounts[0].is_signer = false;
    let err = sim.process_ix(unsigned, &[thief]).unwrap_err();
    assert!(
        err.is_program_error(&ProgramError::MissingRequiredSignature),
        "{err}"
    );
    assert_eq!(sim.lamports(&vault), 2 * SOL);
}

#[test]
fn ex6_a_fake_system_program_is_rejected() {
    let mut sim = sim();
    let user = sim.funded_wallet(3 * SOL);
    let mut ix = ex06_vault::deposit(&user, SOL);
    ix.accounts[2].pubkey = ex03_hello::ID;
    let err = sim.process_ix(ix, &[user]).unwrap_err();
    assert!(
        err.is_program_error(&ProgramError::IncorrectProgramId),
        "{err}"
    );
}

#[test]
fn ex6_cpi_privileges_come_from_the_transaction() {
    // The deposit CPI needs the *user's* signature: invoke can only pass on a
    // signature the transaction has. Asking the vault program to deposit
    // from someone who didn't sign fails in the program's own check, and if
    // the program forgot it, the runtime would refuse the escalation.
    let mut sim = sim();
    let (victim, attacker) = (sim.funded_wallet(3 * SOL), sim.funded_wallet(SOL));
    let mut ix = ex06_vault::deposit(&victim, SOL);
    ix.accounts[0].is_signer = false;
    let err = sim.process_ix(ix, &[attacker]).unwrap_err();
    assert!(
        matches!(
            err.error,
            SimError::Program(_) | SimError::PrivilegeEscalation(_)
        ),
        "{err}"
    );
    assert_eq!(sim.lamports(&victim), 3 * SOL);
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_notes_by_author_filters_and_sorts() {
    let mut sim = sim();
    let (ana, ben) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    for (author, id) in [(ana, 2), (ben, 0), (ana, 0), (ana, 1)] {
        sim.process_ix(
            ex04_notes_state::create_note(&author, id, "t", ""),
            &[author],
        )
        .unwrap();
    }
    let ids: Vec<u64> = bonus_client::notes_by_author(&sim, &ana)
        .iter()
        .map(|n| n.id)
        .collect();
    assert_eq!(ids, [0, 1, 2]);
    assert!(
        bonus_client::notes_by_author(&sim, &ben)
            .iter()
            .all(|n| n.author == ben)
    );
    assert_eq!(bonus_client::next_note_id(&sim, &ana), 3);
    assert_eq!(bonus_client::next_note_id(&sim, &Pubkey::new_unique()), 0);
}

#[test]
fn bonus_memcmp_filter_matches_bytes_at_an_offset() {
    let mut sim = sim();
    let program = Pubkey::new_unique();
    let (a, b, c) = (
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
    );
    sim.set_account(a, Account::new_rent_exempt(vec![1, 2, 3], program));
    sim.set_account(b, Account::new_rent_exempt(vec![9, 2, 3], program));
    sim.set_account(
        c,
        Account::new_rent_exempt(vec![1, 2], Pubkey::new_unique()),
    );
    let mut found = bonus_client::memcmp_filter(&sim, &program, 1, &[2, 3]);
    found.sort();
    let mut expected = vec![a, b];
    expected.sort();
    assert_eq!(found, expected);
    assert!(
        bonus_client::memcmp_filter(&sim, &program, 2, &[3, 4]).is_empty(),
        "out of range doesn't match"
    );
}
