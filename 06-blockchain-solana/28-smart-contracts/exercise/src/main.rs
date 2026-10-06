// 28-smart-contracts -- YOUR WORKSPACE.
//
// This runner is ready to use: it calls the functions in src/ that you
// fill in. Until you implement a part it stops with "not yet implemented".
//
//     cargo run -p m28-smart-contracts -- <1-4|bonus|all>

use borsh::BorshDeserialize;
use m28_smart_contracts::ex04_security::{self as bank, secure_id, vulnerable_id};
use m28_smart_contracts::{
    bonus_governance, ex01_multisig as ms, ex02_escrow as escrow, ex03_staking as staking,
};
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use solsim::{Sim, token};

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(ms::ID, ms::process);
    sim.add_program(escrow::ID, escrow::process);
    sim.add_program(staking::ID, staking::process);
    sim.add_program(vulnerable_id::ID, bank::vulnerable::process);
    sim.add_program(secure_id::ID, bank::secure::process);
    sim
}

fn ex1() {
    println!("--- 1: multisig (2 of 3)");
    let mut sim = sim();
    let owners = [
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    ];
    sim.process_ix(ms::create(&owners[0], 1, &owners, 2), &[owners[0]])
        .unwrap();
    let multisig = ms::multisig_address(&owners[0], 1);
    let vault = ms::vault_address(&multisig);
    sim.airdrop(&vault, 5 * SOL);
    let payee = Pubkey::new_unique();
    let inner = system_instruction::transfer(&vault, &payee, 2 * SOL);
    sim.process_ix(ms::propose(&owners[0], &multisig, 0, &inner), &[owners[0]])
        .unwrap();
    let early = sim
        .process_ix(ms::execute(&owners[0], &multisig, 0, &inner), &[owners[0]])
        .unwrap_err();
    println!(
        "execute with 1 approval: {:?} (NotEnoughApprovals = 4)",
        early.error
    );
    sim.process_ix(ms::approve(&owners[1], &multisig, 0), &[owners[1]])
        .unwrap();
    let meta = sim
        .process_ix(ms::execute(&owners[2], &multisig, 0, &inner), &[owners[2]])
        .unwrap();
    println!(
        "executed after 2 approvals; payee has {} SOL",
        sim.lamports(&payee) / SOL
    );
    for line in meta.logs {
        println!("  {line}");
    }
}

fn ex2() {
    println!("--- 2: escrow");
    let mut sim = sim();
    let (maker, taker, issuer) = (
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    );
    let (mint_a, mint_b) = (
        token::create_mint(&mut sim, &issuer, &issuer, 0),
        token::create_mint(&mut sim, &issuer, &issuer, 0),
    );
    let [maker_a, maker_b, taker_a, taker_b] = [
        (maker, mint_a),
        (maker, mint_b),
        (taker, mint_a),
        (taker, mint_b),
    ]
    .map(|(o, m)| token::create_ata(&mut sim, &issuer, &o, &m));
    token::mint_to(&mut sim, &mint_a, &issuer, &maker_a, 10);
    token::mint_to(&mut sim, &mint_b, &issuer, &taker_b, 25);
    sim.process_ix(
        escrow::make(&maker, 7, &mint_a, &mint_b, &maker_a, &maker_b, 10, 25),
        &[maker],
    )
    .unwrap();
    let escrow_key = escrow::escrow_address(&maker, 7);
    println!(
        "maker offers 10 A for 25 B; vault holds {} A",
        token::balance(&sim, &escrow::vault_address(&escrow_key))
    );
    let redirect = escrow::take(&taker, &maker, &escrow_key, &taker_a, &taker_b, &taker_b);
    println!(
        "taker tries to pay themselves: {:?}",
        sim.process_ix(redirect, &[taker]).unwrap_err().error
    );
    sim.process_ix(
        escrow::take(&taker, &maker, &escrow_key, &taker_a, &taker_b, &maker_b),
        &[taker],
    )
    .unwrap();
    println!(
        "swapped: maker has {} B, taker has {} A; escrow closed: {}",
        token::balance(&sim, &maker_b),
        token::balance(&sim, &taker_a),
        sim.account(&escrow_key).is_none()
    );
}

fn ex3() {
    println!("--- 3: staking (10 reward tokens per second)");
    let mut sim = sim();
    let admin = sim.funded_wallet(SOL);
    let stake_mint = token::create_mint(&mut sim, &admin, &admin, 0);
    let pool = staking::pool_address(&stake_mint);
    let reward_mint = token::create_mint(&mut sim, &admin, &pool, 0);
    sim.process_ix(
        staking::init_pool(&admin, &stake_mint, &reward_mint, 10),
        &[admin],
    )
    .unwrap();
    let mut users = Vec::new();
    for amount in [100, 300] {
        let user = sim.funded_wallet(SOL);
        let tokens = token::create_ata(&mut sim, &admin, &user, &stake_mint);
        token::mint_to(&mut sim, &stake_mint, &admin, &tokens, amount);
        let rewards = token::create_ata(&mut sim, &admin, &user, &reward_mint);
        sim.process_ix(staking::stake(&user, &stake_mint, &tokens, amount), &[user])
            .unwrap();
        users.push((user, rewards, amount));
    }
    sim.advance_time(100);
    for (user, rewards, amount) in users {
        sim.process_ix(
            staking::claim(&user, &stake_mint, &reward_mint, &rewards),
            &[user],
        )
        .unwrap();
        println!(
            "staked {amount:>3} for 100 s -> {} reward tokens",
            token::balance(&sim, &rewards)
        );
    }
    let state = staking::Pool::deserialize(&mut sim.data(&pool)).unwrap();
    println!("reward_per_token = {} / 10^12", state.reward_per_token);
}

fn ex4() {
    println!("--- 4: security review -- bug 6, duplicate mutable accounts");
    for (name, program) in [("vulnerable", vulnerable_id::ID), ("secure", secure_id::ID)] {
        let mut sim = sim();
        let admin = sim.funded_wallet(SOL);
        sim.process_ix(bank::init(&program, &admin), &[admin])
            .unwrap();
        let victim = sim.funded_wallet(20 * SOL);
        sim.process(
            &[
                bank::open(&program, &victim),
                bank::deposit(&program, &victim, 10 * SOL),
            ],
            &[victim],
        )
        .unwrap();
        let attacker = sim.funded_wallet(SOL);
        sim.process(
            &[
                bank::open(&program, &attacker),
                bank::deposit(&program, &attacker, SOL / 2),
            ],
            &[attacker],
        )
        .unwrap();
        let mut balance = SOL / 2;
        let mut result = Ok(());
        while balance < 11 * SOL && result.is_ok() {
            result = sim
                .process_ix(
                    bank::transfer(&program, &attacker, &attacker, balance),
                    &[attacker],
                )
                .map(|_| ());
            balance *= 2;
        }
        match result {
            Ok(()) => {
                let vault = sim.lamports(&bank::vault_address(&program));
                sim.process_ix(bank::withdraw(&program, &attacker, vault), &[attacker])
                    .unwrap();
                println!(
                    "{name}: transferring to itself doubled the balance; attacker now has {:.1} SOL",
                    sim.lamports(&attacker) as f64 / SOL as f64
                );
            }
            Err(e) => println!(
                "{name}: {:?} (SameAccount = 4); the vault keeps {} SOL",
                e.error,
                sim.lamports(&bank::vault_address(&program)) / SOL
            ),
        }
    }
}

fn bonus() {
    println!("--- bonus: the multisig raises its own threshold");
    let mut sim = sim();
    let owners = [
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    ];
    sim.process_ix(ms::create(&owners[0], 1, &owners, 2), &[owners[0]])
        .unwrap();
    let multisig = ms::multisig_address(&owners[0], 1);
    let change = bonus_governance::set_threshold(&multisig, 3);
    sim.process_ix(ms::propose(&owners[0], &multisig, 0, &change), &[owners[0]])
        .unwrap();
    sim.process_ix(ms::approve(&owners[1], &multisig, 0), &[owners[1]])
        .unwrap();
    let meta = sim
        .process_ix(ms::execute(&owners[2], &multisig, 0, &change), &[owners[2]])
        .unwrap();
    let state = ms::Multisig::deserialize(&mut sim.data(&multisig)).unwrap();
    println!(
        "threshold is now {} of {}",
        state.threshold,
        state.owners.len()
    );
    for line in meta.logs {
        println!("  {line}");
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
            "28-smart-contracts -- your workspace\n\n  cargo run -p m28-smart-contracts -- <1-4|bonus|all>"
        ),
    }
}
