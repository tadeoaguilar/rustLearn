// Reference solution for 26-solana-basics.
//
//     cargo run -p m26-solana-basics-solution -- <1-6|bonus|all>

use m26_solana_basics_solution::{
    bonus_client, ex01_wallet, ex02_transfers, ex03_hello, ex04_notes_state, ex05_notes, ex06_vault,
};
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_signer::Signer;
use solsim::Sim;

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(ex03_hello::ID, ex03_hello::process);
    sim.add_program(ex04_notes_state::ID, ex05_notes::process);
    sim.add_program(ex06_vault::ID, ex06_vault::process);
    sim
}

fn ex1() {
    println!("--- 1: wallets");
    let wallet = ex01_wallet::wallet_from_seed(&[7; 32]);
    println!("address (base58 public key): {}", wallet.pubkey());
    let file = ex01_wallet::to_keypair_file(&wallet);
    println!("keypair file: {}...", &file[..40]);
    let again = ex01_wallet::from_keypair_file(&file).unwrap();
    println!(
        "read back the same address: {}",
        again.pubkey() == wallet.pubkey()
    );
    let sig = ex01_wallet::sign_message(&wallet, b"gm");
    println!(
        "signature over \"gm\" verifies: {}",
        ex01_wallet::verify(&wallet.pubkey(), b"gm", &sig)
    );
    println!(
        "...and not over \"gn\": {}",
        ex01_wallet::verify(&wallet.pubkey(), b"gn", &sig)
    );
    let lamports = ex01_wallet::parse_sol("1.25").unwrap();
    println!(
        "1.25 SOL = {lamports} lamports = {}",
        ex01_wallet::format_sol(lamports)
    );
}

fn ex2() {
    println!("--- 2: transfers and rent");
    let mut sim = sim();
    let alice = sim.funded_wallet(2 * LAMPORTS_PER_SOL);
    let (bob, carol) = (Pubkey::new_unique(), Pubkey::new_unique());
    println!(
        "rent-exempt minimum for a wallet: {}",
        ex01_wallet::format_sol(sim.minimum_balance(0))
    );
    println!(
        "alice can spend {}",
        ex01_wallet::format_sol(ex02_transfers::spendable(&sim, &alice))
    );
    ex02_transfers::pay_many(
        &mut sim,
        &alice,
        &[(bob, LAMPORTS_PER_SOL / 2), (carol, LAMPORTS_PER_SOL / 4)],
    )
    .unwrap();
    println!(
        "paid bob and carol in one transaction; alice has {}",
        ex01_wallet::format_sol(sim.lamports(&alice))
    );
    let err = ex02_transfers::pay_many(
        &mut sim,
        &alice,
        &[(bob, 1), (carol, 10 * LAMPORTS_PER_SOL)],
    )
    .unwrap_err();
    println!(
        "a transaction where the 2nd payment is too large fails at instruction {}: {:?}",
        err.index, err.error
    );
    println!(
        "...and bob didn't get the 1 lamport either: {}",
        sim.lamports(&bob) == LAMPORTS_PER_SOL / 2
    );
    let err =
        ex02_transfers::pay_many(&mut sim, &alice, &[(Pubkey::new_unique(), 1000)]).unwrap_err();
    println!(
        "sending 1000 lamports to a new address: {:?} (below rent exemption)",
        err.error
    );
    for key in [alice, ex03_hello::ID, solana_program::sysvar::clock::ID] {
        println!(
            "{key}: {:?}",
            ex02_transfers::describe(sim.account(&key).unwrap())
        );
    }
}

fn ex3() {
    println!("--- 3: hello world");
    let mut sim = sim();
    let payer = sim.funded_wallet(LAMPORTS_PER_SOL);
    let meta = sim
        .process_ix(ex03_hello::hello("Solana", None), &[payer])
        .unwrap();
    let (program, data) = meta.return_data.unwrap();
    println!(
        "return data from {program}: {:?}",
        String::from_utf8(data).unwrap()
    );
    let counter = Pubkey::new_unique();
    // solsim takes the signer list at face value: listing `counter` stands in
    // for its signature, as if we held its keypair.
    ex02_transfers::create_data_account(
        &mut sim,
        &payer,
        &counter,
        ex03_hello::COUNTER_LEN,
        &ex03_hello::ID,
    )
    .unwrap();
    for name in ["Ana", "Ben", "Cy"] {
        sim.process_ix(ex03_hello::hello(name, Some(counter)), &[payer])
            .unwrap();
    }
    println!(
        "greetings counted: {:?}",
        ex03_hello::read_counter(sim.data(&counter))
    );
    for line in meta.logs {
        println!("  log: {line}");
    }
}

fn ex45() {
    println!("--- 4, 5: notes");
    let mut sim = sim();
    let author = sim.funded_wallet(LAMPORTS_PER_SOL);
    sim.process_ix(
        ex04_notes_state::create_note(&author, 0, "gm", "first note"),
        &[author],
    )
    .unwrap();
    let (note, bump) = ex04_notes_state::note_address(&author, 0);
    println!("note 0 lives at the PDA {note} (bump {bump})");
    println!(
        "rent deposit: {}",
        ex01_wallet::format_sol(sim.lamports(&note))
    );
    sim.process_ix(
        ex04_notes_state::update_note(&author, 0, "edited"),
        &[author],
    )
    .unwrap();
    println!(
        "{:?}",
        ex04_notes_state::Note::read(sim.data(&note)).unwrap()
    );
    let mallory = sim.funded_wallet(LAMPORTS_PER_SOL);
    let mut forged = ex04_notes_state::update_note(&author, 0, "pwned");
    forged.accounts[0].pubkey = mallory;
    let err = sim.process_ix(forged, &[mallory]).unwrap_err();
    println!(
        "mallory editing it: {:?} = {:?}",
        err.error,
        ex05_notes::NoteError::NotTheAuthor
    );
    let before = sim.lamports(&author);
    sim.process_ix(ex04_notes_state::delete_note(&author, 0), &[author])
        .unwrap();
    println!(
        "deleted; the account is gone: {}, rent refunded: {}",
        sim.account(&note).is_none(),
        ex01_wallet::format_sol(sim.lamports(&author) - before)
    );
}

fn ex6() {
    println!("--- 6: vault (CPI)");
    let mut sim = sim();
    let user = sim.funded_wallet(3 * LAMPORTS_PER_SOL);
    let (vault, _) = ex06_vault::vault_address(&user);
    let meta = sim
        .process_ix(ex06_vault::deposit(&user, 2 * LAMPORTS_PER_SOL), &[user])
        .unwrap();
    println!(
        "deposited; vault holds {}",
        ex01_wallet::format_sol(sim.lamports(&vault))
    );
    for line in meta.logs {
        println!("  log: {line}");
    }
    sim.process_ix(ex06_vault::withdraw(&user, LAMPORTS_PER_SOL / 2), &[user])
        .unwrap();
    println!(
        "withdrew 0.5 SOL; vault holds {}",
        ex01_wallet::format_sol(sim.lamports(&vault))
    );
    let thief = sim.funded_wallet(LAMPORTS_PER_SOL);
    let mut steal = ex06_vault::withdraw(&thief, LAMPORTS_PER_SOL);
    steal.accounts[1].pubkey = vault;
    println!(
        "a thief pointing withdraw at the user's vault: {:?}",
        sim.process_ix(steal, &[thief]).unwrap_err().error
    );
}

fn bonus() {
    println!("--- bonus: reading program accounts");
    let mut sim = sim();
    let (ana, ben) = (
        sim.funded_wallet(LAMPORTS_PER_SOL),
        sim.funded_wallet(LAMPORTS_PER_SOL),
    );
    for (author, title) in [(ana, "a"), (ben, "b"), (ana, "c")] {
        let id = bonus_client::next_note_id(&sim, &author);
        sim.process_ix(
            ex04_notes_state::create_note(&author, id, title, ""),
            &[author],
        )
        .unwrap();
    }
    for note in bonus_client::notes_by_author(&sim, &ana) {
        println!("ana's note {}: {:?}", note.id, note.title);
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") | Some("5") => ex45(),
        Some("6") => ex6(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex45();
            ex6();
            bonus();
        }
        _ => println!(
            "26-solana-basics -- reference solution\n\n  cargo run -p m26-solana-basics-solution -- <1-6|bonus|all>"
        ),
    }
}
