// Reference solution for 29-nfts-tokens.
//
//     cargo run -p m29-nfts-tokens-solution -- <1-4|bonus|all>

use m29_nfts_tokens_solution::bonus_compressed::{self as cnft, MerkleTree};
use m29_nfts_tokens_solution::ex02_metadata::{self as metadata, Metadata};
use m29_nfts_tokens_solution::ex03_vending::{self as vending, Config};
use m29_nfts_tokens_solution::{ex01_fungible as fungible, ex04_nft_staking as nft_staking};
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::pubkey::Pubkey;
use solsim::{Sim, ata, token};

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(metadata::ID, metadata::process);
    sim.add_program(vending::ID, vending::process);
    sim.add_program(nft_staking::ID, nft_staking::process);
    sim
}

fn ex1() {
    println!("--- 1: a fungible token (6 decimals)");
    let mut sim = sim();
    let (issuer, alice, bob) = (
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        Pubkey::new_unique(),
    );
    let mint = fungible::create_token(&mut sim, &issuer, 6, true).unwrap();
    let amount = fungible::parse_amount("1000", 6).unwrap();
    fungible::mint_to_wallet(&mut sim, &mint, &issuer, &alice, amount).unwrap();
    fungible::send(
        &mut sim,
        &mint,
        &alice,
        &bob,
        fungible::parse_amount("12.5", 6).unwrap(),
    )
    .unwrap();
    fungible::burn(
        &mut sim,
        &mint,
        &alice,
        fungible::parse_amount("0.5", 6).unwrap(),
    )
    .unwrap();
    for (name, who) in [("alice", alice), ("bob", bob)] {
        println!(
            "{name}: {}",
            fungible::format_amount(fungible::balance_of(&sim, &mint, &who), 6)
        );
    }
    println!(
        "supply: {}",
        fungible::format_amount(token::mint_state(&sim, &mint).unwrap().supply, 6)
    );
    fungible::freeze(&mut sim, &mint, &issuer, &bob).unwrap();
    println!(
        "bob frozen, sending to him: {:?}",
        fungible::send(&mut sim, &mint, &alice, &bob, 1)
            .unwrap_err()
            .error
    );
    fungible::fix_supply(&mut sim, &mint, &issuer).unwrap();
    println!(
        "supply fixed, minting more: {:?}",
        fungible::mint_to_wallet(&mut sim, &mint, &issuer, &alice, 1)
            .unwrap_err()
            .error
    );
}

fn ex2() {
    println!("--- 2: an NFT with metadata");
    let mut sim = sim();
    let (artist, collector) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let collection = metadata::mint_nft(
        &mut sim,
        &artist,
        &artist,
        "Rust Crabs",
        "https://example.com/c.json",
        None,
    )
    .unwrap();
    let nft = metadata::mint_nft(
        &mut sim,
        &artist,
        &collector,
        "Crab #1",
        "https://example.com/1.json",
        Some(collection),
    )
    .unwrap();
    let mint = token::mint_state(&sim, &nft).unwrap();
    println!(
        "mint: decimals {}, supply {}, mint authority {:?}",
        mint.decimals, mint.supply, mint.mint_authority
    );
    let read = |sim: &Sim| Metadata::read(sim.data(&metadata::metadata_address(&nft))).unwrap();
    println!(
        "collection verified: {}",
        read(&sim).collection.unwrap().verified
    );
    sim.process_ix(
        metadata::verify_collection(&artist, &nft, &collection),
        &[artist],
    )
    .unwrap();
    let meta = read(&sim);
    println!(
        "{:?} {:?} -> collection verified: {}",
        meta.name,
        meta.uri,
        meta.collection.as_ref().unwrap().verified
    );
    println!(
        "royalties on a 10 SOL sale: {:?}",
        metadata::royalty_split(10 * SOL, &meta)
    );
}

fn ex3() {
    println!("--- 3: a vending machine");
    let mut sim = sim();
    let (creator, treasury) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let config = Config {
        price: SOL / 10,
        items_available: 3,
        go_live: sim.clock().unix_timestamp + 60,
        per_wallet_limit: 2,
        name_prefix: "Gopher".into(),
        base_uri: "https://example.com/gophers".into(),
    };
    sim.process_ix(
        vending::initialize(&creator, 1, &treasury, config),
        &[creator],
    )
    .unwrap();
    let machine = vending::machine_address(&creator, 1);
    let buyer = sim.funded_wallet(SOL);
    let buy = |sim: &mut Sim| {
        let new_mint = Pubkey::new_unique();
        sim.process_ix(
            vending::mint(&buyer, &machine, &treasury, &new_mint),
            &[buyer, new_mint],
        )
        .map(|_| new_mint)
    };
    println!("before go-live: {:?}", buy(&mut sim).unwrap_err().error);
    sim.advance_time(60);
    for _ in 0..2 {
        let nft = buy(&mut sim).unwrap();
        let meta = Metadata::read(sim.data(&metadata::metadata_address(&nft))).unwrap();
        let owned = token::balance(&sim, &ata::get_associated_token_address(&buyer, &nft));
        println!("bought {:?} ({}), owns {owned}", meta.name, meta.uri);
    }
    println!(
        "a third for the same wallet: {:?}",
        buy(&mut sim).unwrap_err().error
    );
    println!(
        "treasury earned {} SOL",
        (sim.lamports(&treasury) - SOL) as f64 / SOL as f64
    );
}

fn ex4() {
    println!("--- 4: NFT staking (2 reward tokens per second)");
    let mut sim = sim();
    let (artist, holder) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let collection = metadata::mint_nft(&mut sim, &artist, &artist, "Crabs", "c", None).unwrap();
    let nft =
        metadata::mint_nft(&mut sim, &artist, &holder, "Crab #7", "7", Some(collection)).unwrap();
    sim.process_ix(
        metadata::verify_collection(&artist, &nft, &collection),
        &[artist],
    )
    .unwrap();
    let reward_mint = token::create_mint(
        &mut sim,
        &artist,
        &nft_staking::farm_address(&collection),
        0,
    );
    sim.process_ix(
        nft_staking::init_farm(&artist, &collection, &reward_mint, 2),
        &[artist],
    )
    .unwrap();
    let holder_nft = ata::get_associated_token_address(&holder, &nft);
    let holder_rewards = token::create_ata(&mut sim, &holder, &holder, &reward_mint);
    sim.process_ix(
        nft_staking::stake(&holder, &collection, &nft, &holder_nft),
        &[holder],
    )
    .unwrap();
    println!(
        "staked; holder's NFT balance: {}",
        token::balance(&sim, &holder_nft)
    );
    sim.advance_time(3600);
    sim.process_ix(
        nft_staking::unstake(
            &holder,
            &collection,
            &nft,
            &holder_nft,
            &reward_mint,
            &holder_rewards,
        ),
        &[holder],
    )
    .unwrap();
    println!(
        "unstaked after an hour: NFT back ({}), {} reward tokens",
        token::balance(&sim, &holder_nft),
        token::balance(&sim, &holder_rewards)
    );
    let fake =
        metadata::mint_nft(&mut sim, &holder, &holder, "Crab #8", "8", Some(collection)).unwrap();
    let fake_account = ata::get_associated_token_address(&holder, &fake);
    let err = sim
        .process_ix(
            nft_staking::stake(&holder, &collection, &fake, &fake_account),
            &[holder],
        )
        .unwrap_err();
    println!(
        "an NFT claiming the collection, unverified: {:?}",
        err.error
    );
}

fn bonus() {
    println!("--- bonus: compressed NFTs (a depth-3 tree: 8 leaves)");
    let mut tree = MerkleTree::new(3);
    let owners: Vec<Pubkey> = (0..8).map(|_| Pubkey::new_unique()).collect();
    for (i, owner) in owners.iter().enumerate() {
        tree.set(i, cnft::leaf_hash(owner, &format!("Leaf #{i}"), "uri"));
    }
    let root = tree.root();
    let leaf = cnft::leaf_hash(&owners[5], "Leaf #5", "uri");
    let proof = tree.proof(5);
    println!(
        "proof for leaf 5: {} hashes; verifies: {}",
        proof.len(),
        cnft::verify(&root, &leaf, 5, &proof)
    );
    let new_owner = Pubkey::new_unique();
    let moved = cnft::leaf_hash(&new_owner, "Leaf #5", "uri");
    let new_root = cnft::replace_leaf(&root, &leaf, &moved, 5, &proof).unwrap();
    tree.set(5, moved);
    println!(
        "transferred: on-chain root updated from the proof alone matches the tree: {}",
        new_root == tree.root()
    );
    println!(
        "replaying the old proof against the new root: {:?}",
        cnft::replace_leaf(&new_root, &leaf, &moved, 5, &proof)
    );
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
            "29-nfts-tokens -- reference solution\n\n  cargo run -p m29-nfts-tokens-solution -- <1-4|bonus|all>"
        ),
    }
}
