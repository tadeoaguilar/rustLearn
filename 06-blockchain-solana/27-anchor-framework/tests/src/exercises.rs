use crate::sut::{
    self as lab, Counter, Poll, Profile, Registry, VoteReceipt, errors::LabError, ex02_voting,
    ex03_registry, ex04_client,
};
use anchor_lang::error::ErrorCode as AnchorError;
use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData, Space, ToAccountMetas};
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solsim::{Account, Sim, TxError, TxMeta};

const SYSTEM: Pubkey = anchor_lang::system_program::ID;

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(lab::ID, lab::entry);
    sim
}

fn ix(accounts: impl ToAccountMetas, data: impl InstructionData) -> Instruction {
    Instruction {
        program_id: lab::ID,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}

fn send(
    sim: &mut Sim,
    accounts: impl ToAccountMetas,
    data: impl InstructionData,
    signer: Pubkey,
) -> Result<TxMeta, TxError> {
    sim.process_ix(ix(accounts, data), &[signer])
}

fn load<T: AccountDeserialize>(sim: &Sim, key: &Pubkey) -> T {
    T::try_deserialize(&mut sim.data(key)).expect("account of the expected type")
}

fn pda(seeds: &[&[u8]]) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, &lab::ID)
}

#[track_caller]
fn assert_lab_error(result: Result<TxMeta, TxError>, expected: LabError) {
    let err = result.expect_err("should fail");
    assert_eq!(
        err.custom_code(),
        Some(u32::from(expected)),
        "expected {expected:?}, got {err}"
    );
}

#[track_caller]
fn assert_anchor_error(result: Result<TxMeta, TxError>, expected: &[AnchorError]) {
    let err = result.expect_err("should fail");
    let codes: Vec<u32> = expected.iter().map(|e| u32::from(*e)).collect();
    assert!(
        err.custom_code().is_some_and(|c| codes.contains(&c)),
        "expected one of {codes:?}, got {err}"
    );
}

// ---------------------------------------------------------------- Exercise 1

fn counter_of(authority: &Pubkey) -> Pubkey {
    pda(&[b"counter", authority.as_ref()]).0
}

fn init_counter(sim: &mut Sim, authority: Pubkey) -> Result<TxMeta, TxError> {
    let accounts = lab::accounts::InitializeCounter {
        authority,
        counter: counter_of(&authority),
        system_program: SYSTEM,
    };
    send(
        sim,
        accounts,
        lab::instruction::InitializeCounter {},
        authority,
    )
}

fn update(
    sim: &mut Sim,
    authority: Pubkey,
    counter: Pubkey,
    data: impl InstructionData,
) -> Result<TxMeta, TxError> {
    send(
        sim,
        lab::accounts::UpdateCounter { authority, counter },
        data,
        authority,
    )
}

fn count(sim: &Sim, authority: &Pubkey) -> u64 {
    load::<Counter>(sim, &counter_of(authority)).count
}

#[test]
fn ex1_initialize_creates_the_counter_pda() {
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    init_counter(&mut sim, alice).unwrap();
    let (key, bump) = pda(&[b"counter", alice.as_ref()]);
    let account = sim.account(&key).expect("counter account");
    assert_eq!(account.owner, lab::ID);
    assert_eq!(
        account.data.len(),
        8 + 32 + 8 + 1,
        "discriminator + Counter"
    );
    let counter: Counter = load(&sim, &key);
    assert_eq!(
        (counter.authority, counter.count, counter.bump),
        (alice, 0, bump)
    );
    assert!(
        init_counter(&mut sim, alice).is_err(),
        "only one counter per authority"
    );
}

#[test]
fn ex1_increment_decrement_reset() {
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    init_counter(&mut sim, alice).unwrap();
    let counter = counter_of(&alice);
    update(
        &mut sim,
        alice,
        counter,
        lab::instruction::Increment { by: 10 },
    )
    .unwrap();
    update(
        &mut sim,
        alice,
        counter,
        lab::instruction::Decrement { by: 3 },
    )
    .unwrap();
    assert_eq!(count(&sim, &alice), 7);
    update(&mut sim, alice, counter, lab::instruction::Reset {}).unwrap();
    assert_eq!(count(&sim, &alice), 0);
}

#[test]
fn ex1_underflow_and_overflow_are_errors() {
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    init_counter(&mut sim, alice).unwrap();
    let counter = counter_of(&alice);
    assert_lab_error(
        update(
            &mut sim,
            alice,
            counter,
            lab::instruction::Decrement { by: 1 },
        ),
        LabError::CounterUnderflow,
    );
    update(
        &mut sim,
        alice,
        counter,
        lab::instruction::Increment { by: u64::MAX },
    )
    .unwrap();
    assert_lab_error(
        update(
            &mut sim,
            alice,
            counter,
            lab::instruction::Increment { by: 1 },
        ),
        LabError::CounterOverflow,
    );
}

#[test]
fn ex1_only_the_authority_changes_its_counter() {
    let mut sim = sim();
    let (alice, bob) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    init_counter(&mut sim, alice).unwrap();
    let result = update(
        &mut sim,
        bob,
        counter_of(&alice),
        lab::instruction::Increment { by: 1 },
    );
    assert_anchor_error(
        result,
        &[AnchorError::ConstraintSeeds, AnchorError::ConstraintHasOne],
    );

    // alice's key, without alice's signature
    let mut unsigned = ix(
        lab::accounts::UpdateCounter {
            authority: alice,
            counter: counter_of(&alice),
        },
        lab::instruction::Reset {},
    );
    unsigned.accounts[0].is_signer = false;
    let result = sim.process_ix(unsigned, &[bob]);
    assert_anchor_error(result, &[AnchorError::AccountNotSigner]);
}

#[test]
fn ex1_close_refunds_the_rent() {
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    init_counter(&mut sim, alice).unwrap();
    let counter = counter_of(&alice);
    let (rent, before) = (sim.lamports(&counter), sim.lamports(&alice));
    let bob = sim.funded_wallet(SOL);
    let stolen = lab::accounts::CloseCounter {
        authority: bob,
        counter,
    };
    assert!(send(&mut sim, stolen, lab::instruction::CloseCounter {}, bob).is_err());

    send(
        &mut sim,
        lab::accounts::CloseCounter {
            authority: alice,
            counter,
        },
        lab::instruction::CloseCounter {},
        alice,
    )
    .unwrap();
    assert!(sim.account(&counter).is_none());
    assert_eq!(sim.lamports(&alice), before + rent);
    init_counter(&mut sim, alice).unwrap();
}

// ---------------------------------------------------------------- Exercise 2

fn poll_of(creator: &Pubkey, poll_id: u64) -> Pubkey {
    pda(&[b"poll", creator.as_ref(), &poll_id.to_le_bytes()]).0
}

fn create_poll(
    sim: &mut Sim,
    creator: Pubkey,
    poll_id: u64,
    question: &str,
    options: &[&str],
    ends_at: i64,
) -> Result<TxMeta, TxError> {
    let accounts = lab::accounts::CreatePoll {
        creator,
        poll: poll_of(&creator, poll_id),
        system_program: SYSTEM,
    };
    let data = lab::instruction::CreatePoll {
        poll_id,
        question: question.into(),
        options: options.iter().map(|o| o.to_string()).collect(),
        ends_at,
    };
    send(sim, accounts, data, creator)
}

fn vote(sim: &mut Sim, voter: Pubkey, poll: Pubkey, option: u8) -> Result<TxMeta, TxError> {
    let receipt = pda(&[b"vote", poll.as_ref(), voter.as_ref()]).0;
    send(
        sim,
        lab::accounts::CastVote {
            voter,
            poll,
            receipt,
            system_program: SYSTEM,
        },
        lab::instruction::Vote { option },
        voter,
    )
}

fn standard_poll(sim: &mut Sim) -> (Pubkey, Pubkey, i64) {
    let creator = sim.funded_wallet(SOL);
    let ends_at = sim.clock().unix_timestamp + 600;
    create_poll(
        sim,
        creator,
        7,
        "Best editor?",
        &["vim", "emacs", "helix"],
        ends_at,
    )
    .unwrap();
    (creator, poll_of(&creator, 7), ends_at)
}

#[test]
fn ex2_create_poll_stores_everything() {
    let mut sim = sim();
    let (creator, poll, ends_at) = standard_poll(&mut sim);
    let state: Poll = load(&sim, &poll);
    assert_eq!(
        (state.creator, state.poll_id, state.ends_at),
        (creator, 7, ends_at)
    );
    assert_eq!(state.question, "Best editor?");
    assert_eq!(state.options, ["vim", "emacs", "helix"]);
    assert_eq!(state.votes, [0, 0, 0]);
    assert_eq!(
        state.bump,
        pda(&[b"poll", creator.as_ref(), &7u64.to_le_bytes()]).1
    );
    assert_eq!(
        sim.data(&poll).len(),
        8 + Poll::INIT_SPACE,
        "sized for the largest poll"
    );
}

#[test]
fn ex2_create_poll_validates() {
    let mut sim = sim();
    let creator = sim.funded_wallet(SOL);
    let later = sim.clock().unix_timestamp + 60;
    let long_q = "q".repeat(101);
    let long_o = "o".repeat(33);
    assert_lab_error(
        create_poll(&mut sim, creator, 1, "", &["a", "b"], later),
        LabError::BadQuestion,
    );
    assert_lab_error(
        create_poll(&mut sim, creator, 1, &long_q, &["a", "b"], later),
        LabError::BadQuestion,
    );
    assert_lab_error(
        create_poll(&mut sim, creator, 1, "q", &["a"], later),
        LabError::BadOptions,
    );
    assert_lab_error(
        create_poll(&mut sim, creator, 1, "q", &["a", "b", "c", "d", "e"], later),
        LabError::BadOptions,
    );
    assert_lab_error(
        create_poll(&mut sim, creator, 1, "q", &["a", ""], later),
        LabError::BadOptions,
    );
    assert_lab_error(
        create_poll(&mut sim, creator, 1, "q", &["a", &long_o], later),
        LabError::BadOptions,
    );
    let now = sim.clock().unix_timestamp;
    assert_lab_error(
        create_poll(&mut sim, creator, 1, "q", &["a", "b"], now),
        LabError::EndsInThePast,
    );
    create_poll(
        &mut sim,
        creator,
        1,
        &"q".repeat(100),
        &["a", "b", "c", &"o".repeat(32)],
        later,
    )
    .unwrap();
}

#[test]
fn ex2_votes_are_tallied_with_a_receipt_each() {
    let mut sim = sim();
    let (_, poll, _) = standard_poll(&mut sim);
    let voters: Vec<Pubkey> = (0..3).map(|_| sim.funded_wallet(SOL)).collect();
    for (voter, option) in voters.iter().zip([2, 2, 0]) {
        vote(&mut sim, *voter, poll, option).unwrap();
    }
    assert_eq!(load::<Poll>(&sim, &poll).votes, [1, 0, 2]);
    let receipt_key = pda(&[b"vote", poll.as_ref(), voters[0].as_ref()]).0;
    let receipt: VoteReceipt = load(&sim, &receipt_key);
    assert_eq!(
        (receipt.poll, receipt.voter, receipt.option),
        (poll, voters[0], 2)
    );
}

#[test]
fn ex2_one_vote_per_voter() {
    let mut sim = sim();
    let (_, poll, _) = standard_poll(&mut sim);
    let voter = sim.funded_wallet(SOL);
    vote(&mut sim, voter, poll, 0).unwrap();
    assert!(vote(&mut sim, voter, poll, 1).is_err());
    assert_eq!(load::<Poll>(&sim, &poll).votes, [1, 0, 0]);

    // and the receipt must be the voter's own PDA
    let other = sim.funded_wallet(SOL);
    let someone_elses = pda(&[b"vote", poll.as_ref(), Pubkey::new_unique().as_ref()]).0;
    let accounts = lab::accounts::CastVote {
        voter: other,
        poll,
        receipt: someone_elses,
        system_program: SYSTEM,
    };
    let result = send(
        &mut sim,
        accounts,
        lab::instruction::Vote { option: 0 },
        other,
    );
    assert_anchor_error(result, &[AnchorError::ConstraintSeeds]);
}

#[test]
fn ex2_no_such_option_and_no_late_votes() {
    let mut sim = sim();
    let (_, poll, _) = standard_poll(&mut sim);
    let voter = sim.funded_wallet(SOL);
    assert_lab_error(vote(&mut sim, voter, poll, 3), LabError::NoSuchOption);
    sim.advance_time(600);
    assert_lab_error(vote(&mut sim, voter, poll, 0), LabError::PollEnded);
}

#[test]
fn ex2_close_poll_after_it_ends() {
    let mut sim = sim();
    let (creator, poll, _) = standard_poll(&mut sim);
    let close = |sim: &mut Sim, who: Pubkey| {
        send(
            sim,
            lab::accounts::ClosePoll { creator: who, poll },
            lab::instruction::ClosePoll {},
            who,
        )
    };
    assert_lab_error(close(&mut sim, creator), LabError::PollRunning);
    sim.advance_time(601);
    let stranger = sim.funded_wallet(SOL);
    assert_anchor_error(close(&mut sim, stranger), &[AnchorError::ConstraintHasOne]);
    let (rent, before) = (sim.lamports(&poll), sim.lamports(&creator));
    close(&mut sim, creator).unwrap();
    assert!(sim.account(&poll).is_none());
    assert_eq!(sim.lamports(&creator), before + rent);
}

#[test]
fn ex2_winners_include_ties() {
    let poll = |votes: Vec<u64>| Poll {
        creator: Pubkey::default(),
        poll_id: 0,
        question: String::new(),
        options: vec!["a".into(), "b".into(), "c".into()],
        votes,
        ends_at: 0,
        bump: 0,
    };
    assert_eq!(ex02_voting::winners(&poll(vec![1, 5, 2])), ["b"]);
    assert_eq!(ex02_voting::winners(&poll(vec![3, 1, 3])), ["a", "c"]);
    assert_eq!(ex02_voting::winners(&poll(vec![0, 0, 0])), ["a", "b", "c"]);
}

// ---------------------------------------------------------------- Exercise 3

fn registry() -> Pubkey {
    pda(&[b"registry"]).0
}

fn profile_of(owner: &Pubkey) -> Pubkey {
    pda(&[b"profile", owner.as_ref()]).0
}

fn init_registry(sim: &mut Sim) -> Pubkey {
    let admin = sim.funded_wallet(SOL);
    let accounts = lab::accounts::InitializeRegistry {
        admin,
        registry: registry(),
        system_program: SYSTEM,
    };
    send(
        sim,
        accounts,
        lab::instruction::InitializeRegistry {},
        admin,
    )
    .unwrap();
    admin
}

fn create_profile(sim: &mut Sim, owner: Pubkey, handle: &str) -> Result<TxMeta, TxError> {
    let accounts = lab::accounts::CreateProfile {
        owner,
        registry: registry(),
        profile: profile_of(&owner),
        system_program: SYSTEM,
    };
    send(
        sim,
        accounts,
        lab::instruction::CreateProfile {
            handle: handle.into(),
        },
        owner,
    )
}

fn verify(sim: &mut Sim, admin: Pubkey, profile: Pubkey) -> Result<TxMeta, TxError> {
    send(
        sim,
        lab::accounts::VerifyProfile {
            admin,
            registry: registry(),
            profile,
        },
        lab::instruction::VerifyProfile {},
        admin,
    )
}

#[test]
fn ex3_valid_handles() {
    for ok in ["abc", "carol_dev", "a1b2c3", "sixteen_chars_ok"] {
        assert!(ex03_registry::valid_handle(ok), "{ok}");
    }
    for bad in [
        "ab",
        "seventeen_chars__",
        "Carol",
        "car ol",
        "café",
        "dash-y",
    ] {
        assert!(!ex03_registry::valid_handle(bad), "{bad}");
    }
}

#[test]
fn ex3_registry_and_profiles() {
    let mut sim = sim();
    let admin = init_registry(&mut sim);
    let reg: Registry = load(&sim, &registry());
    assert_eq!(
        (reg.admin, reg.profiles, reg.bump),
        (admin, 0, pda(&[b"registry"]).1)
    );

    let carol = sim.funded_wallet(SOL);
    create_profile(&mut sim, carol, "carol").unwrap();
    let profile: Profile = load(&sim, &profile_of(&carol));
    assert_eq!(
        (
            profile.owner,
            profile.handle.as_str(),
            profile.verified,
            profile.bio.as_str()
        ),
        (carol, "carol", false, "")
    );
    assert_eq!(
        sim.data(&profile_of(&carol)).len(),
        8 + Profile::space(5, 0),
        "sized for its handle"
    );
    assert_eq!(load::<Registry>(&sim, &registry()).profiles, 1);

    assert!(
        create_profile(&mut sim, carol, "carol2").is_err(),
        "one profile per wallet"
    );
    let dave = sim.funded_wallet(SOL);
    assert_lab_error(create_profile(&mut sim, dave, "Dave!"), LabError::BadHandle);
}

#[test]
fn ex3_only_the_admin_verifies_once() {
    let mut sim = sim();
    let admin = init_registry(&mut sim);
    let carol = sim.funded_wallet(SOL);
    create_profile(&mut sim, carol, "carol").unwrap();
    assert_lab_error(
        verify(&mut sim, carol, profile_of(&carol)),
        LabError::NotAdmin,
    );
    verify(&mut sim, admin, profile_of(&carol)).unwrap();
    assert!(load::<Profile>(&sim, &profile_of(&carol)).verified);
    assert_lab_error(
        verify(&mut sim, admin, profile_of(&carol)),
        LabError::AlreadyVerified,
    );
}

#[test]
fn ex3_transfer_admin() {
    let mut sim = sim();
    let admin = init_registry(&mut sim);
    let (new_admin, carol) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let transfer = |sim: &mut Sim, by: Pubkey| {
        send(
            sim,
            lab::accounts::TransferAdmin {
                admin: by,
                registry: registry(),
            },
            lab::instruction::TransferAdmin { new_admin },
            by,
        )
    };
    assert_lab_error(transfer(&mut sim, carol), LabError::NotAdmin);
    transfer(&mut sim, admin).unwrap();
    create_profile(&mut sim, carol, "carol").unwrap();
    assert_lab_error(
        verify(&mut sim, admin, profile_of(&carol)),
        LabError::NotAdmin,
    );
    verify(&mut sim, new_admin, profile_of(&carol)).unwrap();
}

#[test]
fn ex3_type_confusion_is_caught() {
    let mut sim = sim();
    let admin = init_registry(&mut sim);
    // The registry passed where a profile is expected: wrong discriminator.
    assert_anchor_error(
        verify(&mut sim, admin, registry()),
        &[AnchorError::AccountDiscriminatorMismatch],
    );

    // A perfect Profile, discriminator and all -- but owned by another program.
    let fake = Pubkey::new_unique();
    let profile = Profile {
        owner: admin,
        handle: "fake".into(),
        verified: false,
        bio: String::new(),
        bump: 255,
    };
    let mut data = Vec::new();
    profile.try_serialize(&mut data).unwrap();
    sim.set_account(fake, Account::new_rent_exempt(data, Pubkey::new_unique()));
    assert_anchor_error(
        verify(&mut sim, admin, fake),
        &[AnchorError::AccountOwnedByWrongProgram],
    );
}

#[test]
fn ex3_delete_profile() {
    let mut sim = sim();
    init_registry(&mut sim);
    let (carol, mallory) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    create_profile(&mut sim, carol, "carol").unwrap();
    let delete = |sim: &mut Sim, owner: Pubkey, profile: Pubkey| {
        send(
            sim,
            lab::accounts::DeleteProfile {
                owner,
                registry: registry(),
                profile,
            },
            lab::instruction::DeleteProfile {},
            owner,
        )
    };
    assert!(delete(&mut sim, mallory, profile_of(&carol)).is_err());
    let rent = sim.lamports(&profile_of(&carol));
    let before = sim.lamports(&carol);
    delete(&mut sim, carol, profile_of(&carol)).unwrap();
    assert!(sim.account(&profile_of(&carol)).is_none());
    assert_eq!(sim.lamports(&carol), before + rent);
    assert_eq!(load::<Registry>(&sim, &registry()).profiles, 0);
}

// ---------------------------------------------------------------- Exercise 4

#[test]
fn ex4_addresses() {
    let (a, b) = (Pubkey::new_unique(), Pubkey::new_unique());
    assert_eq!(ex04_client::counter_address(&a), counter_of(&a));
    assert_eq!(ex04_client::poll_address(&a, 3), poll_of(&a, 3));
    assert_eq!(
        ex04_client::receipt_address(&a, &b),
        pda(&[b"vote", a.as_ref(), b.as_ref()]).0
    );
    assert_eq!(ex04_client::registry_address(), registry());
    assert_eq!(ex04_client::profile_address(&a), profile_of(&a));
}

#[test]
fn ex4_builders_match_anchors_structs() {
    let (a, b) = (Pubkey::new_unique(), Pubkey::new_unique());
    let expected = ix(
        lab::accounts::InitializeCounter {
            authority: a,
            counter: counter_of(&a),
            system_program: SYSTEM,
        },
        lab::instruction::InitializeCounter {},
    );
    assert_eq!(ex04_client::initialize_counter_ix(&a), expected);
    let expected = ix(
        lab::accounts::UpdateCounter {
            authority: a,
            counter: counter_of(&a),
        },
        lab::instruction::Increment { by: 9 },
    );
    assert_eq!(ex04_client::increment_ix(&a, 9), expected);
    let poll = poll_of(&a, 1);
    let expected = ix(
        lab::accounts::CreatePoll {
            creator: a,
            poll,
            system_program: SYSTEM,
        },
        lab::instruction::CreatePoll {
            poll_id: 1,
            question: "q".into(),
            options: vec!["x".into(), "y".into()],
            ends_at: 5,
        },
    );
    assert_eq!(
        ex04_client::create_poll_ix(&a, 1, "q", &["x", "y"], 5),
        expected
    );
    let receipt = pda(&[b"vote", poll.as_ref(), b.as_ref()]).0;
    let expected = ix(
        lab::accounts::CastVote {
            voter: b,
            poll,
            receipt,
            system_program: SYSTEM,
        },
        lab::instruction::Vote { option: 1 },
    );
    assert_eq!(ex04_client::vote_ix(&b, &poll, 1), expected);
    let expected = ix(
        lab::accounts::CreateProfile {
            owner: a,
            registry: registry(),
            profile: profile_of(&a),
            system_program: SYSTEM,
        },
        lab::instruction::CreateProfile { handle: "h".into() },
    );
    assert_eq!(ex04_client::create_profile_ix(&a, "h"), expected);
    let expected = ix(
        lab::accounts::VerifyProfile {
            admin: a,
            registry: registry(),
            profile: profile_of(&b),
        },
        lab::instruction::VerifyProfile {},
    );
    assert_eq!(ex04_client::verify_profile_ix(&a, &b), expected);
}

#[test]
fn ex4_fetch_checks_owner_and_type() {
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    sim.process_ix(ex04_client::initialize_counter_ix(&alice), &[alice])
        .unwrap();
    sim.process_ix(ex04_client::increment_ix(&alice, 4), &[alice])
        .unwrap();
    let key = ex04_client::counter_address(&alice);
    assert_eq!(
        ex04_client::fetch::<Counter>(&sim, &key).map(|c| c.count),
        Some(4)
    );
    assert!(
        ex04_client::fetch::<Poll>(&sim, &key).is_none(),
        "wrong type"
    );
    assert!(
        ex04_client::fetch::<Counter>(&sim, &Pubkey::new_unique()).is_none(),
        "missing"
    );
    let copy = Pubkey::new_unique();
    sim.set_account(
        copy,
        Account::new_rent_exempt(sim.data(&key).to_vec(), Pubkey::new_unique()),
    );
    assert!(
        ex04_client::fetch::<Counter>(&sim, &copy).is_none(),
        "wrong owner"
    );
}

#[test]
fn ex4_lab_error_decodes_codes() {
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    let err = sim
        .process_ix(
            ex04_client::create_poll_ix(&alice, 0, "q", &["one"], i64::MAX),
            &[alice],
        )
        .unwrap_err();
    assert!(matches!(
        ex04_client::lab_error(&err),
        Some(LabError::BadOptions)
    ));
    for (i, e) in ex04_client::ALL_ERRORS.iter().enumerate() {
        assert_eq!(u32::from(*e), 6000 + i as u32);
    }
    // not a LabError: the System program's AccountAlreadyInUse
    sim.process_ix(ex04_client::initialize_counter_ix(&alice), &[alice])
        .unwrap();
    let err = sim
        .process_ix(ex04_client::initialize_counter_ix(&alice), &[alice])
        .unwrap_err();
    assert!(ex04_client::lab_error(&err).is_none());
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_set_bio_reallocates() {
    let mut sim = sim();
    init_registry(&mut sim);
    let dan = sim.funded_wallet(SOL);
    create_profile(&mut sim, dan, "dan").unwrap();
    let profile = profile_of(&dan);
    let set_bio = |sim: &mut Sim, owner: Pubkey, bio: &str| {
        send(
            sim,
            lab::accounts::SetBio {
                owner,
                profile,
                system_program: SYSTEM,
            },
            lab::instruction::SetBio { bio: bio.into() },
            owner,
        )
    };
    for bio in ["short", &"long ".repeat(30), ""] {
        let wallet_before = sim.lamports(&dan) + sim.lamports(&profile);
        set_bio(&mut sim, dan, bio).unwrap();
        let account = sim.account(&profile).unwrap();
        assert_eq!(account.data.len(), 8 + Profile::space(3, bio.len()));
        assert_eq!(
            account.lamports,
            sim.minimum_balance(account.data.len()),
            "exactly rent-exempt"
        );
        assert_eq!(
            sim.lamports(&dan) + account.lamports,
            wallet_before,
            "lamports moved, not created"
        );
        assert_eq!(load::<Profile>(&sim, &profile).bio, bio);
    }
    assert_lab_error(
        set_bio(&mut sim, dan, &"x".repeat(201)),
        LabError::BioTooLong,
    );
    let mallory = sim.funded_wallet(SOL);
    assert!(set_bio(&mut sim, mallory, "pwned").is_err());
}
