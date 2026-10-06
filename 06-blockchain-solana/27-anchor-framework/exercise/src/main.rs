// 27-anchor-framework -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m27-anchor-framework -- <1-4|bonus|all>

use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use m27_anchor_framework::{
    self as lab, Counter, Poll, Profile, Registry, ex02_voting, ex04_client as client,
};
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solsim::{Sim, TxError, TxMeta};

const SYSTEM: Pubkey = anchor_lang::system_program::ID;

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(lab::ID, lab::entry);
    sim
}

/// Send one instruction built from Anchor's generated structs.
fn send(
    sim: &mut Sim,
    accounts: impl ToAccountMetas,
    data: impl InstructionData,
    signer: Pubkey,
) -> Result<TxMeta, TxError> {
    let ix = Instruction {
        program_id: lab::ID,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    };
    sim.process_ix(ix, &[signer])
}

fn load<T: AccountDeserialize>(sim: &Sim, key: &Pubkey) -> T {
    T::try_deserialize(&mut sim.data(key)).unwrap()
}

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &lab::ID).0
}

fn ex1() {
    println!("--- 1: counter");
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    let counter = pda(&[b"counter", alice.as_ref()]);
    let meta = send(
        &mut sim,
        lab::accounts::InitializeCounter {
            authority: alice,
            counter,
            system_program: SYSTEM,
        },
        lab::instruction::InitializeCounter {},
        alice,
    )
    .unwrap();
    for line in &meta.logs {
        println!("  {line}");
    }
    send(
        &mut sim,
        lab::accounts::UpdateCounter {
            authority: alice,
            counter,
        },
        lab::instruction::Increment { by: 5 },
        alice,
    )
    .unwrap();
    println!(
        "alice's counter at {counter}: {}",
        load::<Counter>(&sim, &counter).count
    );
    let err = send(
        &mut sim,
        lab::accounts::UpdateCounter {
            authority: alice,
            counter,
        },
        lab::instruction::Decrement { by: 6 },
        alice,
    )
    .unwrap_err();
    println!(
        "decrement by 6: {:?} (6000 = LabError::CounterUnderflow)",
        err.error
    );

    let bob = sim.funded_wallet(SOL);
    let err = send(
        &mut sim,
        lab::accounts::UpdateCounter {
            authority: bob,
            counter,
        },
        lab::instruction::Increment { by: 1 },
        bob,
    )
    .unwrap_err();
    println!(
        "bob incrementing alice's counter: {:?} (Anchor's ConstraintSeeds is 2006)",
        err.error
    );
}

fn ex2() {
    println!("--- 2: voting");
    let mut sim = sim();
    let creator = sim.funded_wallet(SOL);
    let poll = pda(&[b"poll", creator.as_ref(), &1u64.to_le_bytes()]);
    let ends_at = sim.clock().unix_timestamp + 3600;
    let create = lab::instruction::CreatePoll {
        poll_id: 1,
        question: "Tabs or spaces?".into(),
        options: vec!["tabs".into(), "spaces".into()],
        ends_at,
    };
    send(
        &mut sim,
        lab::accounts::CreatePoll {
            creator,
            poll,
            system_program: SYSTEM,
        },
        create,
        creator,
    )
    .unwrap();
    let vote = |sim: &mut Sim, voter: Pubkey, option: u8| {
        let receipt = pda(&[b"vote", poll.as_ref(), voter.as_ref()]);
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
    };
    for (i, option) in [1, 1, 0].into_iter().enumerate() {
        let voter = sim.funded_wallet(SOL);
        vote(&mut sim, voter, option).unwrap();
        if i == 0 {
            let err = vote(&mut sim, voter, 0).unwrap_err();
            println!(
                "voting twice: {:?} (the System program: the receipt account already exists)",
                err.error
            );
        }
    }
    let state: Poll = load(&sim, &poll);
    println!(
        "{:?} {:?} -> {:?}",
        state.options,
        state.votes,
        ex02_voting::winners(&state)
    );
    sim.advance_time(3601);
    let late = sim.funded_wallet(SOL);
    println!(
        "voting after the end: {:?} (6006 = PollEnded)",
        vote(&mut sim, late, 0).unwrap_err().error
    );
}

fn init_registry(sim: &mut Sim) -> Pubkey {
    let admin = sim.funded_wallet(SOL);
    let registry = pda(&[b"registry"]);
    send(
        sim,
        lab::accounts::InitializeRegistry {
            admin,
            registry,
            system_program: SYSTEM,
        },
        lab::instruction::InitializeRegistry {},
        admin,
    )
    .unwrap();
    admin
}

fn ex3() {
    println!("--- 3: registry");
    let mut sim = sim();
    let admin = init_registry(&mut sim);
    let registry = pda(&[b"registry"]);
    let carol = sim.funded_wallet(SOL);
    let profile = pda(&[b"profile", carol.as_ref()]);
    let create = lab::accounts::CreateProfile {
        owner: carol,
        registry,
        profile,
        system_program: SYSTEM,
    };
    send(
        &mut sim,
        create,
        lab::instruction::CreateProfile {
            handle: "carol_dev".into(),
        },
        carol,
    )
    .unwrap();
    let err = send(
        &mut sim,
        lab::accounts::VerifyProfile {
            admin: carol,
            registry,
            profile,
        },
        lab::instruction::VerifyProfile {},
        carol,
    )
    .unwrap_err();
    println!("carol verifying herself: {:?} (6009 = NotAdmin)", err.error);
    send(
        &mut sim,
        lab::accounts::VerifyProfile {
            admin,
            registry,
            profile,
        },
        lab::instruction::VerifyProfile {},
        admin,
    )
    .unwrap();
    let (p, r): (Profile, Registry) = (load(&sim, &profile), load(&sim, &registry));
    println!(
        "@{} verified: {}; profiles: {}",
        p.handle, p.verified, r.profiles
    );
    // Type confusion: pass the registry where a profile is expected.
    let err = send(
        &mut sim,
        lab::accounts::VerifyProfile {
            admin,
            registry,
            profile: registry,
        },
        lab::instruction::VerifyProfile {},
        admin,
    )
    .unwrap_err();
    println!(
        "the registry passed as a profile: {:?} (3002 = AccountDiscriminatorMismatch)",
        err.error
    );
}

fn ex4() {
    println!("--- 4: the client");
    let mut sim = sim();
    let alice = sim.funded_wallet(SOL);
    sim.process_ix(client::initialize_counter_ix(&alice), &[alice])
        .unwrap();
    sim.process_ix(client::increment_ix(&alice, 3), &[alice])
        .unwrap();
    let counter: Option<Counter> = client::fetch(&sim, &client::counter_address(&alice));
    println!("fetched: count = {:?}", counter.map(|c| c.count));
    let wrong_type: Option<Poll> = client::fetch(&sim, &client::counter_address(&alice));
    println!(
        "fetching the counter as a Poll: {:?}",
        wrong_type.map(|p| p.poll_id)
    );
    let err = sim
        .process_ix(
            client::create_poll_ix(&alice, 0, "?", &["only one"], i64::MAX),
            &[alice],
        )
        .unwrap_err();
    println!("a poll with one option: {:?}", client::lab_error(&err));
}

fn bonus() {
    println!("--- bonus: realloc");
    let mut sim = sim();
    init_registry(&mut sim);
    let dan = sim.funded_wallet(SOL);
    let profile = pda(&[b"profile", dan.as_ref()]);
    let create = lab::accounts::CreateProfile {
        owner: dan,
        registry: pda(&[b"registry"]),
        profile,
        system_program: SYSTEM,
    };
    send(
        &mut sim,
        create,
        lab::instruction::CreateProfile {
            handle: "dan".into(),
        },
        dan,
    )
    .unwrap();
    for bio in [
        "Rustacean",
        "Rustacean. Writes Solana programs and long bios about them.",
        "",
    ] {
        let accounts = lab::accounts::SetBio {
            owner: dan,
            profile,
            system_program: SYSTEM,
        };
        send(
            &mut sim,
            accounts,
            lab::instruction::SetBio { bio: bio.into() },
            dan,
        )
        .unwrap();
        let account = sim.account(&profile).unwrap();
        println!(
            "bio of {:>2} bytes -> account {:>3} bytes, {} lamports",
            bio.len(),
            account.data.len(),
            account.lamports
        );
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") => ex4(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex4();
            bonus();
        }
        _ => println!(
            "27-anchor-framework -- your workspace\n\n  cargo run -p m27-anchor-framework -- <1-4|bonus|all>"
        ),
    }
}
