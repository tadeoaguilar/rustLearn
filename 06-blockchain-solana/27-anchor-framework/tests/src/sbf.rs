//! The on-chain track: the program built by `./build-sbf.sh`, in LiteSVM.

use crate::WHICH;
use crate::sut::{self as lab, Counter, Poll, Profile};
use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_keypair::Keypair;
use solana_program::instruction::Instruction;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_signer::Signer;
use solsim::sbf::{Vm, VmError, VmMeta};

const SYSTEM: Pubkey = anchor_lang::system_program::ID;

fn vm() -> Vm {
    let mut vm = Vm::new();
    vm.load(lab::ID, WHICH, "anchor_lab");
    vm
}

fn send(
    vm: &mut Vm,
    accounts: impl ToAccountMetas,
    data: impl InstructionData,
    signer: &Keypair,
) -> Result<VmMeta, VmError> {
    let ix = Instruction {
        program_id: lab::ID,
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    };
    vm.process(&[ix], &[signer])
}

fn load<T: AccountDeserialize>(vm: &Vm, key: &Pubkey) -> T {
    T::try_deserialize(&mut vm.data(key).as_slice()).unwrap()
}

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &lab::ID).0
}

#[test]
fn sbf_counter() {
    let mut vm = vm();
    let alice = vm.wallet(SOL);
    let a = alice.pubkey();
    let counter = pda(&[b"counter", a.as_ref()]);
    send(
        &mut vm,
        lab::accounts::InitializeCounter {
            authority: a,
            counter,
            system_program: SYSTEM,
        },
        lab::instruction::InitializeCounter {},
        &alice,
    )
    .unwrap();
    let meta = send(
        &mut vm,
        lab::accounts::UpdateCounter {
            authority: a,
            counter,
        },
        lab::instruction::Increment { by: 2 },
        &alice,
    )
    .unwrap();
    println!("increment used {} compute units", meta.compute_units);
    assert_eq!(load::<Counter>(&vm, &counter).count, 2);
    let err = send(
        &mut vm,
        lab::accounts::UpdateCounter {
            authority: a,
            counter,
        },
        lab::instruction::Decrement { by: 3 },
        &alice,
    )
    .unwrap_err();
    assert_eq!(err.custom_code(), Some(6000));
}

#[test]
fn sbf_poll() {
    let mut vm = vm();
    let creator = vm.wallet(SOL);
    let c = creator.pubkey();
    let poll = pda(&[b"poll", c.as_ref(), &1u64.to_le_bytes()]);
    let create = lab::instruction::CreatePoll {
        poll_id: 1,
        question: "q".into(),
        options: vec!["a".into(), "b".into()],
        ends_at: i64::MAX,
    };
    send(
        &mut vm,
        lab::accounts::CreatePoll {
            creator: c,
            poll,
            system_program: SYSTEM,
        },
        create,
        &creator,
    )
    .unwrap();
    let voter = vm.wallet(SOL);
    let v = voter.pubkey();
    let receipt = pda(&[b"vote", poll.as_ref(), v.as_ref()]);
    let accounts = || lab::accounts::CastVote {
        voter: v,
        poll,
        receipt,
        system_program: SYSTEM,
    };
    send(
        &mut vm,
        accounts(),
        lab::instruction::Vote { option: 1 },
        &voter,
    )
    .unwrap();
    assert!(
        send(
            &mut vm,
            accounts(),
            lab::instruction::Vote { option: 0 },
            &voter
        )
        .is_err()
    );
    assert_eq!(load::<Poll>(&vm, &poll).votes, [0, 1]);
}

#[test]
fn sbf_profile_realloc() {
    let mut vm = vm();
    let admin = vm.wallet(SOL);
    let registry = pda(&[b"registry"]);
    send(
        &mut vm,
        lab::accounts::InitializeRegistry {
            admin: admin.pubkey(),
            registry,
            system_program: SYSTEM,
        },
        lab::instruction::InitializeRegistry {},
        &admin,
    )
    .unwrap();
    let dan = vm.wallet(SOL);
    let d = dan.pubkey();
    let profile = pda(&[b"profile", d.as_ref()]);
    send(
        &mut vm,
        lab::accounts::CreateProfile {
            owner: d,
            registry,
            profile,
            system_program: SYSTEM,
        },
        lab::instruction::CreateProfile {
            handle: "dan".into(),
        },
        &dan,
    )
    .unwrap();
    let bio = "on the real VM";
    send(
        &mut vm,
        lab::accounts::SetBio {
            owner: d,
            profile,
            system_program: SYSTEM,
        },
        lab::instruction::SetBio { bio: bio.into() },
        &dan,
    )
    .unwrap();
    assert_eq!(load::<Profile>(&vm, &profile).bio, bio);
    assert_eq!(vm.data(&profile).len(), 8 + Profile::space(3, bio.len()));
}
