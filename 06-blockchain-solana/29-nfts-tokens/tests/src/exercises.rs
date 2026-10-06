use crate::sut::bonus_compressed::{self as cnft, MerkleTree};
use crate::sut::ex01_fungible as fungible;
use crate::sut::ex02_metadata::{self as metadata, CreateArgs, Metadata, MetadataError};
use crate::sut::ex03_vending::{self as vending, Config, VendingError};
use crate::sut::ex04_nft_staking::{self as nft_staking, NftStakingError};
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::pubkey::Pubkey;
use solsim::{Sim, TxError, TxMeta, ata, token};

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(metadata::ID, metadata::process);
    sim.add_program(vending::ID, vending::process);
    sim.add_program(nft_staking::ID, nft_staking::process);
    sim
}

#[track_caller]
fn assert_error(result: Result<TxMeta, TxError>, expected: impl Into<ProgramError>) {
    let expected = expected.into();
    let err = result.expect_err("should fail");
    assert!(
        err.is_program_error(&expected),
        "expected {expected:?}, got {err}"
    );
}

/// SPL Token's own error codes.
const TOKEN_INSUFFICIENT_FUNDS: u32 = 1;
const TOKEN_FIXED_SUPPLY: u32 = 5;
const TOKEN_ACCOUNT_FROZEN: u32 = 17;

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_create_mint_ixs() {
    let (payer, mint, auth) = (
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
    );
    let [create, init] = fungible::create_mint_ixs(&payer, &mint, &auth, None, 9, 1234);
    assert_eq!(create.program_id, solana_sdk_ids::system_program::ID);
    assert_eq!(
        (create.accounts[0].pubkey, create.accounts[1].pubkey),
        (payer, mint)
    );
    assert_eq!(init.program_id, spl_token_interface::ID);
    let expected = spl_token_interface::instruction::initialize_mint2(
        &spl_token_interface::ID,
        &mint,
        &auth,
        None,
        9,
    )
    .unwrap();
    assert_eq!(init, expected);
}

#[test]
fn ex1_create_token_and_mint() {
    let mut sim = sim();
    let (issuer, alice) = (sim.funded_wallet(SOL), Pubkey::new_unique());
    let mint = fungible::create_token(&mut sim, &issuer, 6, true).unwrap();
    let state = token::mint_state(&sim, &mint).unwrap();
    assert_eq!((state.decimals, state.supply), (6, 0));
    assert_eq!(state.mint_authority, COption::Some(issuer));
    assert_eq!(state.freeze_authority, COption::Some(issuer));
    assert_eq!(fungible::decimals(&sim, &mint), 6);

    let ata = fungible::mint_to_wallet(&mut sim, &mint, &issuer, &alice, 5_000_000).unwrap();
    assert_eq!(ata, ata::get_associated_token_address(&alice, &mint));
    assert_eq!(fungible::balance_of(&sim, &mint, &alice), 5_000_000);
    fungible::mint_to_wallet(&mut sim, &mint, &issuer, &alice, 1).unwrap(); // the ATA exists: still fine
    assert_eq!(token::mint_state(&sim, &mint).unwrap().supply, 5_000_001);
    let not_freezable = fungible::create_token(&mut sim, &issuer, 0, false).unwrap();
    assert_eq!(
        token::mint_state(&sim, &not_freezable)
            .unwrap()
            .freeze_authority,
        COption::None
    );
}

#[test]
fn ex1_send_creates_the_recipients_account() {
    let mut sim = sim();
    let (issuer, alice, bob) = (
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        Pubkey::new_unique(),
    );
    let mint = fungible::create_token(&mut sim, &issuer, 2, false).unwrap();
    fungible::mint_to_wallet(&mut sim, &mint, &issuer, &alice, 1000).unwrap();
    fungible::send(&mut sim, &mint, &alice, &bob, 250).unwrap();
    assert_eq!(
        (
            fungible::balance_of(&sim, &mint, &alice),
            fungible::balance_of(&sim, &mint, &bob)
        ),
        (750, 250)
    );

    // too much: nothing happens, not even the recipient's account
    let carol = Pubkey::new_unique();
    let err = fungible::send(&mut sim, &mint, &alice, &carol, 751).unwrap_err();
    assert_eq!(err.custom_code(), Some(TOKEN_INSUFFICIENT_FUNDS));
    assert!(
        sim.account(&ata::get_associated_token_address(&carol, &mint))
            .is_none()
    );
}

#[test]
fn ex1_burn_freeze_thaw_fix_supply() {
    let mut sim = sim();
    let (issuer, alice, bob) = (
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    );
    let mint = fungible::create_token(&mut sim, &issuer, 0, true).unwrap();
    fungible::mint_to_wallet(&mut sim, &mint, &issuer, &alice, 100).unwrap();
    fungible::burn(&mut sim, &mint, &alice, 40).unwrap();
    assert_eq!(token::mint_state(&sim, &mint).unwrap().supply, 60);

    fungible::send(&mut sim, &mint, &alice, &bob, 10).unwrap();
    fungible::freeze(&mut sim, &mint, &issuer, &bob).unwrap();
    assert_eq!(
        fungible::send(&mut sim, &mint, &alice, &bob, 1)
            .unwrap_err()
            .custom_code(),
        Some(TOKEN_ACCOUNT_FROZEN)
    );
    assert_eq!(
        fungible::send(&mut sim, &mint, &bob, &alice, 1)
            .unwrap_err()
            .custom_code(),
        Some(TOKEN_ACCOUNT_FROZEN)
    );
    assert!(
        fungible::freeze(&mut sim, &mint, &alice, &bob).is_err(),
        "only the freeze authority"
    );
    fungible::thaw(&mut sim, &mint, &issuer, &bob).unwrap();
    fungible::send(&mut sim, &mint, &bob, &alice, 1).unwrap();

    fungible::fix_supply(&mut sim, &mint, &issuer).unwrap();
    assert_eq!(
        token::mint_state(&sim, &mint).unwrap().mint_authority,
        COption::None
    );
    let err = fungible::mint_to_wallet(&mut sim, &mint, &issuer, &alice, 1).unwrap_err();
    assert_eq!(err.custom_code(), Some(TOKEN_FIXED_SUPPLY));
}

#[test]
fn ex1_amounts() {
    assert_eq!(fungible::format_amount(1_500_000, 6), "1.5");
    assert_eq!(fungible::format_amount(1, 6), "0.000001");
    assert_eq!(fungible::format_amount(42, 0), "42");
    assert_eq!(fungible::format_amount(3_000_000_000, 9), "3");
    assert_eq!(fungible::parse_amount("1.5", 6), Ok(1_500_000));
    assert_eq!(fungible::parse_amount("7", 0), Ok(7));
    assert_eq!(fungible::parse_amount("0.000001", 6), Ok(1));
    for bad in ["1.0000001", "1.5", "", "x", "-1", "99999999999999999999"] {
        let decimals = if bad == "1.5" { 0 } else { 6 };
        assert!(
            fungible::parse_amount(bad, decimals).is_err(),
            "{bad:?} with {decimals} decimals"
        );
    }
}

// ---------------------------------------------------------------- Exercise 2

fn read_meta(sim: &Sim, mint: &Pubkey) -> Metadata {
    Metadata::read(sim.data(&metadata::metadata_address(mint))).expect("metadata")
}

fn args(name: &str) -> CreateArgs {
    CreateArgs {
        name: name.into(),
        symbol: "SYM".into(),
        uri: "https://example.com/x.json".into(),
        seller_fee_basis_points: 250,
        creators: vec![],
        collection: None,
        is_mutable: true,
    }
}

#[test]
fn ex2_mint_nft_makes_a_one_of_one() {
    let mut sim = sim();
    let (artist, owner) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let nft = metadata::mint_nft(
        &mut sim,
        &artist,
        &owner,
        "Crab",
        "https://e.com/1.json",
        None,
    )
    .unwrap();
    let mint = token::mint_state(&sim, &nft).unwrap();
    assert_eq!(
        (mint.decimals, mint.supply, mint.mint_authority),
        (0, 1, COption::None)
    );
    assert_eq!(
        token::balance(&sim, &ata::get_associated_token_address(&owner, &nft)),
        1
    );
    let meta = read_meta(&sim, &nft);
    assert_eq!((meta.mint, meta.update_authority), (nft, artist));
    assert_eq!(
        (meta.name.as_str(), meta.uri.as_str()),
        ("Crab", "https://e.com/1.json")
    );
    assert_eq!(
        meta.creators,
        [metadata::Creator {
            address: artist,
            share: 100,
            verified: true
        }]
    );
    assert_eq!(
        sim.account(&metadata::metadata_address(&nft))
            .unwrap()
            .owner,
        metadata::ID
    );
}

#[test]
fn ex2_create_validates() {
    let mut sim = sim();
    let authority = sim.funded_wallet(SOL);
    let mint = token::create_mint(&mut sim, &authority, &authority, 0);
    let create = |sim: &mut Sim, a: CreateArgs| {
        sim.process_ix(
            metadata::create(&authority, &mint, &authority, a),
            &[authority],
        )
    };
    assert_error(
        create(&mut sim, args(&"n".repeat(33))),
        MetadataError::TooLong,
    );
    assert_error(
        create(
            &mut sim,
            CreateArgs {
                uri: "u".repeat(201),
                ..args("n")
            },
        ),
        MetadataError::TooLong,
    );
    assert_error(
        create(
            &mut sim,
            CreateArgs {
                seller_fee_basis_points: 10_001,
                ..args("n")
            },
        ),
        MetadataError::BadRoyalty,
    );
    let (a, b) = (Pubkey::new_unique(), Pubkey::new_unique());
    assert_error(
        create(
            &mut sim,
            CreateArgs {
                creators: vec![(a, 50), (b, 40)],
                ..args("n")
            },
        ),
        MetadataError::BadCreators,
    );
    let six = (0..6).map(|_| (Pubkey::new_unique(), 0)).collect();
    assert_error(
        create(
            &mut sim,
            CreateArgs {
                creators: six,
                ..args("n")
            },
        ),
        MetadataError::BadCreators,
    );
    create(
        &mut sim,
        CreateArgs {
            creators: vec![(a, 60), (b, 40)],
            ..args("n")
        },
    )
    .unwrap();
    assert!(
        create(&mut sim, args("again")).is_err(),
        "one metadata per mint"
    );
    let meta = read_meta(&sim, &mint);
    assert!(
        meta.creators.iter().all(|c| !c.verified),
        "creators who didn't sign aren't verified"
    );
}

#[test]
fn ex2_only_the_mint_authority_creates_metadata() {
    let mut sim = sim();
    let (authority, impostor) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let mint = token::create_mint(&mut sim, &authority, &authority, 0);
    let result = sim.process_ix(
        metadata::create(&impostor, &mint, &impostor, args("mine now")),
        &[impostor],
    );
    assert_error(result, MetadataError::NotMintAuthority);
}

#[test]
fn ex2_update_and_immutability() {
    let mut sim = sim();
    let (artist, other) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let nft = metadata::mint_nft(&mut sim, &artist, &artist, "Old", "old", None).unwrap();
    let result = sim.process_ix(
        metadata::update(&other, &nft, Some("Hacked".into()), None, None, None),
        &[other],
    );
    assert_error(result, MetadataError::NotUpdateAuthority);
    sim.process_ix(
        metadata::update(
            &artist,
            &nft,
            Some("New".into()),
            Some("new".into()),
            None,
            Some(false),
        ),
        &[artist],
    )
    .unwrap();
    let meta = read_meta(&sim, &nft);
    assert_eq!(
        (meta.name.as_str(), meta.uri.as_str(), meta.is_mutable),
        ("New", "new", false)
    );
    let result = sim.process_ix(
        metadata::update(&artist, &nft, Some("Again".into()), None, None, None),
        &[artist],
    );
    assert_error(result, MetadataError::Immutable);
}

#[test]
fn ex2_handing_over_the_update_authority() {
    let mut sim = sim();
    let (artist, heir) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let nft = metadata::mint_nft(&mut sim, &artist, &artist, "N", "u", None).unwrap();
    sim.process_ix(
        metadata::update(&artist, &nft, None, None, Some(heir), None),
        &[artist],
    )
    .unwrap();
    assert_error(
        sim.process_ix(
            metadata::update(&artist, &nft, None, Some("x".into()), None, None),
            &[artist],
        ),
        MetadataError::NotUpdateAuthority,
    );
    sim.process_ix(
        metadata::update(&heir, &nft, None, Some("x".into()), None, None),
        &[heir],
    )
    .unwrap();
}

#[test]
fn ex2_creators_verify_themselves() {
    let mut sim = sim();
    let (authority, cocreator, stranger) = (
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    );
    let mint = token::create_mint(&mut sim, &authority, &authority, 0);
    let a = CreateArgs {
        creators: vec![(authority, 70), (cocreator, 30)],
        ..args("Duo")
    };
    sim.process_ix(
        metadata::create(&authority, &mint, &authority, a),
        &[authority],
    )
    .unwrap();
    let verified = |sim: &Sim| {
        read_meta(sim, &mint)
            .creators
            .iter()
            .map(|c| c.verified)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        verified(&sim),
        [true, false],
        "the signing mint authority is verified at creation"
    );
    assert_error(
        sim.process_ix(metadata::verify_creator(&stranger, &mint), &[stranger]),
        MetadataError::NotACreator,
    );
    sim.process_ix(metadata::verify_creator(&cocreator, &mint), &[cocreator])
        .unwrap();
    assert_eq!(verified(&sim), [true, true]);
}

#[test]
fn ex2_collections_are_verified_by_their_authority() {
    let mut sim = sim();
    let (artist, forger) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let collection =
        metadata::mint_nft(&mut sim, &artist, &artist, "Collection", "c", None).unwrap();
    let member =
        metadata::mint_nft(&mut sim, &artist, &artist, "Member", "m", Some(collection)).unwrap();
    let fake =
        metadata::mint_nft(&mut sim, &forger, &forger, "Member", "m", Some(collection)).unwrap();
    assert!(!read_meta(&sim, &member).collection.unwrap().verified);

    sim.process_ix(
        metadata::verify_collection(&artist, &member, &collection),
        &[artist],
    )
    .unwrap();
    assert!(read_meta(&sim, &member).collection.unwrap().verified);
    let result = sim.process_ix(
        metadata::verify_collection(&forger, &fake, &collection),
        &[forger],
    );
    assert_error(result, MetadataError::NotUpdateAuthority);
    // verifying against a collection the NFT doesn't name
    let other = metadata::mint_nft(&mut sim, &forger, &forger, "Other", "o", None).unwrap();
    assert_error(
        sim.process_ix(
            metadata::verify_collection(&forger, &fake, &other),
            &[forger],
        ),
        MetadataError::WrongCollection,
    );
}

#[test]
fn ex2_royalty_split() {
    let creators = vec![
        metadata::Creator {
            address: Pubkey::new_from_array([1; 32]),
            share: 70,
            verified: true,
        },
        metadata::Creator {
            address: Pubkey::new_from_array([2; 32]),
            share: 30,
            verified: false,
        },
    ];
    let meta = Metadata {
        mint: Pubkey::default(),
        update_authority: Pubkey::default(),
        name: String::new(),
        symbol: String::new(),
        uri: String::new(),
        seller_fee_basis_points: 500,
        creators,
        collection: None,
        is_mutable: true,
        bump: 0,
    };
    let split = metadata::royalty_split(1_000_000, &meta);
    assert_eq!(
        split,
        [
            (Pubkey::new_from_array([1; 32]), 35_000),
            (Pubkey::new_from_array([2; 32]), 15_000)
        ]
    );
    assert_eq!(
        metadata::royalty_split(u64::MAX, &meta)[0].1,
        (u64::MAX as u128 * 500 / 10_000 * 70 / 100) as u64
    );
}

// ---------------------------------------------------------------- Exercise 3

fn machine(sim: &mut Sim, items: u32, limit: u8) -> (Pubkey, Pubkey) {
    let (creator, treasury) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let config = Config {
        price: SOL / 4,
        items_available: items,
        go_live: sim.clock().unix_timestamp,
        per_wallet_limit: limit,
        name_prefix: "Ferris".into(),
        base_uri: "https://e.com/f".into(),
    };
    sim.process_ix(
        vending::initialize(&creator, 9, &treasury, config),
        &[creator],
    )
    .unwrap();
    (vending::machine_address(&creator, 9), treasury)
}

fn buy(
    sim: &mut Sim,
    buyer: &Pubkey,
    machine: &Pubkey,
    treasury: &Pubkey,
) -> Result<Pubkey, TxError> {
    let new_mint = Pubkey::new_unique();
    sim.process_ix(
        vending::mint(buyer, machine, treasury, &new_mint),
        &[*buyer, new_mint],
    )
    .map(|_| new_mint)
}

#[test]
fn ex3_initialize_stores_the_config() {
    let mut sim = sim();
    let (machine, treasury) = machine(&mut sim, 5, 1);
    let state: vending::Machine =
        borsh::BorshDeserialize::deserialize(&mut sim.data(&machine)).unwrap();
    assert_eq!(
        (
            state.treasury,
            state.items_redeemed,
            state.config.items_available
        ),
        (treasury, 0, 5)
    );
    let creator = sim.funded_wallet(SOL);
    let too_long = Config {
        name_prefix: "x".repeat(25),
        ..state.config.clone()
    };
    assert_error(
        sim.process_ix(
            vending::initialize(&creator, 1, &treasury, too_long),
            &[creator],
        ),
        VendingError::BadConfig,
    );
}

#[test]
fn ex3_buying_mints_a_numbered_nft() {
    let mut sim = sim();
    let (machine, treasury) = machine(&mut sim, 5, 3);
    let buyer = sim.funded_wallet(SOL);
    let before = sim.lamports(&treasury);
    let first = buy(&mut sim, &buyer, &machine, &treasury).unwrap();
    let second = buy(&mut sim, &buyer, &machine, &treasury).unwrap();
    assert_eq!(sim.lamports(&treasury), before + SOL / 2);
    for (n, nft) in [(1, first), (2, second)] {
        let mint = token::mint_state(&sim, &nft).unwrap();
        assert_eq!(
            (mint.decimals, mint.supply, mint.mint_authority),
            (0, 1, COption::None),
            "a 1-of-1 nobody can mint again"
        );
        assert_eq!(
            token::balance(&sim, &ata::get_associated_token_address(&buyer, &nft)),
            1
        );
        let meta = read_meta(&sim, &nft);
        assert_eq!(meta.name, format!("Ferris #{n}"));
        assert_eq!(meta.uri, format!("https://e.com/f/{n}.json"));
        assert_eq!(meta.update_authority, machine);
    }
    let state: vending::Machine =
        borsh::BorshDeserialize::deserialize(&mut sim.data(&machine)).unwrap();
    assert_eq!(state.items_redeemed, 2);
}

#[test]
fn ex3_limits() {
    let mut sim = sim();
    let (creator, treasury) = (sim.funded_wallet(SOL), sim.funded_wallet(SOL));
    let config = Config {
        price: 1000,
        items_available: 2,
        go_live: sim.clock().unix_timestamp + 100,
        per_wallet_limit: 1,
        name_prefix: "N".into(),
        base_uri: "u".into(),
    };
    sim.process_ix(
        vending::initialize(&creator, 1, &treasury, config),
        &[creator],
    )
    .unwrap();
    let machine = vending::machine_address(&creator, 1);
    let (a, b, c) = (
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    );
    assert_error(
        buy(&mut sim, &a, &machine, &treasury).map(|_| TxMeta::default()),
        VendingError::NotLive,
    );
    sim.advance_time(100);
    buy(&mut sim, &a, &machine, &treasury).unwrap();
    assert_error(
        buy(&mut sim, &a, &machine, &treasury).map(|_| TxMeta::default()),
        VendingError::WalletLimit,
    );
    buy(&mut sim, &b, &machine, &treasury).unwrap();
    assert_error(
        buy(&mut sim, &c, &machine, &treasury).map(|_| TxMeta::default()),
        VendingError::SoldOut,
    );
}

#[test]
fn ex3_payments_and_programs_cannot_be_redirected() {
    let mut sim = sim();
    let (machine, treasury) = machine(&mut sim, 5, 5);
    let buyer = sim.funded_wallet(SOL);
    let new_mint = Pubkey::new_unique();
    let mut to_self = vending::mint(&buyer, &machine, &treasury, &new_mint);
    to_self.accounts[2].pubkey = buyer;
    assert_error(
        sim.process_ix(to_self, &[buyer, new_mint]),
        VendingError::WrongTreasury,
    );
    let mut fake_metadata = vending::mint(&buyer, &machine, &treasury, &new_mint);
    fake_metadata.accounts[10].pubkey = nft_staking::ID;
    assert_error(
        sim.process_ix(fake_metadata, &[buyer, new_mint]),
        VendingError::WrongProgram,
    );
}

// ---------------------------------------------------------------- Exercise 4

struct Farm {
    artist: Pubkey,
    collection: Pubkey,
    reward_mint: Pubkey,
}

fn farm(sim: &mut Sim, rate: u64) -> Farm {
    let artist = sim.funded_wallet(10 * SOL);
    let collection = metadata::mint_nft(sim, &artist, &artist, "Collection", "c", None).unwrap();
    let reward_mint = token::create_mint(sim, &artist, &nft_staking::farm_address(&collection), 0);
    sim.process_ix(
        nft_staking::init_farm(&artist, &collection, &reward_mint, rate),
        &[artist],
    )
    .unwrap();
    Farm {
        artist,
        collection,
        reward_mint,
    }
}

/// A verified member NFT owned by a new holder; (holder, mint, holder's NFT account).
fn member(sim: &mut Sim, f: &Farm) -> (Pubkey, Pubkey, Pubkey) {
    let holder = sim.funded_wallet(SOL);
    let nft =
        metadata::mint_nft(sim, &f.artist, &holder, "Member", "m", Some(f.collection)).unwrap();
    sim.process_ix(
        metadata::verify_collection(&f.artist, &nft, &f.collection),
        &[f.artist],
    )
    .unwrap();
    (
        holder,
        nft,
        ata::get_associated_token_address(&holder, &nft),
    )
}

#[test]
fn ex4_init_farm_requires_it_to_mint_rewards() {
    let mut sim = sim();
    let artist = sim.funded_wallet(SOL);
    let collection = metadata::mint_nft(&mut sim, &artist, &artist, "C", "c", None).unwrap();
    let reward_mint = token::create_mint(&mut sim, &artist, &artist, 0);
    let result = sim.process_ix(
        nft_staking::init_farm(&artist, &collection, &reward_mint, 1),
        &[artist],
    );
    assert_error(result, NftStakingError::InvalidRewardMint);
}

#[test]
fn ex4_stake_and_unstake_with_rewards() {
    let mut sim = sim();
    let f = farm(&mut sim, 3);
    let (holder, nft, holder_nft) = member(&mut sim, &f);
    let rewards = token::create_ata(&mut sim, &holder, &holder, &f.reward_mint);
    sim.process_ix(
        nft_staking::stake(&holder, &f.collection, &nft, &holder_nft),
        &[holder],
    )
    .unwrap();
    let vault = nft_staking::vault_address(&nft);
    assert_eq!(
        (
            token::balance(&sim, &holder_nft),
            token::balance(&sim, &vault)
        ),
        (0, 1)
    );
    assert_eq!(
        token::token_account_state(&sim, &vault).unwrap().owner,
        nft_staking::farm_address(&f.collection)
    );
    let record: nft_staking::StakeRecord =
        borsh::BorshDeserialize::deserialize(&mut sim.data(&nft_staking::record_address(&nft)))
            .unwrap();
    assert_eq!(
        (record.owner, record.mint, record.staked_at),
        (holder, nft, sim.clock().unix_timestamp)
    );

    sim.advance_time(100);
    sim.process_ix(
        nft_staking::unstake(
            &holder,
            &f.collection,
            &nft,
            &holder_nft,
            &f.reward_mint,
            &rewards,
        ),
        &[holder],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &holder_nft), 1);
    assert_eq!(token::balance(&sim, &rewards), 300);
    assert!(
        sim.account(&vault).is_none() && sim.account(&nft_staking::record_address(&nft)).is_none()
    );
}

#[test]
fn ex4_only_verified_members_qualify() {
    let mut sim = sim();
    let f = farm(&mut sim, 1);
    // claims the collection, but nobody verified it
    let holder = sim.funded_wallet(SOL);
    let claimed = metadata::mint_nft(
        &mut sim,
        &holder,
        &holder,
        "Member",
        "m",
        Some(f.collection),
    )
    .unwrap();
    let claimed_account = ata::get_associated_token_address(&holder, &claimed);
    assert_error(
        sim.process_ix(
            nft_staking::stake(&holder, &f.collection, &claimed, &claimed_account),
            &[holder],
        ),
        NftStakingError::NotInCollection,
    );
    // verified -- in another collection
    let other = farm(&mut sim, 1);
    let (outsider, their_nft, their_account) = member(&mut sim, &other);
    assert_error(
        sim.process_ix(
            nft_staking::stake(&outsider, &f.collection, &their_nft, &their_account),
            &[outsider],
        ),
        NftStakingError::NotInCollection,
    );
    // a member's metadata passed for a non-member
    let (_, real_member, _) = member(&mut sim, &f);
    let mut borrowed = nft_staking::stake(&holder, &f.collection, &claimed, &claimed_account);
    borrowed.accounts[3].pubkey = metadata::metadata_address(&real_member);
    assert_error(
        sim.process_ix(borrowed, &[holder]),
        NftStakingError::WrongMetadata,
    );
}

#[test]
fn ex4_only_the_staker_unstakes() {
    let mut sim = sim();
    let f = farm(&mut sim, 1);
    let (holder, nft, holder_nft) = member(&mut sim, &f);
    sim.process_ix(
        nft_staking::stake(&holder, &f.collection, &nft, &holder_nft),
        &[holder],
    )
    .unwrap();
    let thief = sim.funded_wallet(SOL);
    let thief_nft = token::create_ata(&mut sim, &thief, &thief, &nft);
    let thief_rewards = token::create_ata(&mut sim, &thief, &thief, &f.reward_mint);
    let result = sim.process_ix(
        nft_staking::unstake(
            &thief,
            &f.collection,
            &nft,
            &thief_nft,
            &f.reward_mint,
            &thief_rewards,
        ),
        &[thief],
    );
    assert_error(result, NftStakingError::NotTheStaker);
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_proofs_verify_only_their_leaf() {
    let mut tree = MerkleTree::new(4);
    let owners: Vec<Pubkey> = (0..16).map(|_| Pubkey::new_unique()).collect();
    for (i, o) in owners.iter().enumerate() {
        tree.set(i, cnft::leaf_hash(o, "n", "u"));
    }
    let root = tree.root();
    for i in [0, 7, 15] {
        let leaf = cnft::leaf_hash(&owners[i], "n", "u");
        let proof = tree.proof(i);
        assert_eq!(proof.len(), 4);
        assert!(cnft::verify(&root, &leaf, i, &proof));
        assert!(!cnft::verify(&root, &leaf, i ^ 1, &proof), "wrong position");
        let impostor = cnft::leaf_hash(&Pubkey::new_unique(), "n", "u");
        assert!(!cnft::verify(&root, &impostor, i, &proof), "wrong owner");
    }
    assert_ne!(
        cnft::leaf_hash(&owners[0], "ab", "c"),
        cnft::leaf_hash(&owners[0], "a", "bc"),
        "fields can't run together"
    );
}

#[test]
fn bonus_replace_leaf_is_a_transfer() {
    let mut tree = MerkleTree::new(3);
    let alice = Pubkey::new_unique();
    let leaf = cnft::leaf_hash(&alice, "n", "u");
    tree.set(2, leaf);
    let root = tree.root();
    let proof = tree.proof(2);
    let to_bob = cnft::leaf_hash(&Pubkey::new_unique(), "n", "u");
    let new_root = cnft::replace_leaf(&root, &leaf, &to_bob, 2, &proof).unwrap();
    tree.set(2, to_bob);
    assert_eq!(new_root, tree.root());
    assert_eq!(
        cnft::replace_leaf(&new_root, &leaf, &to_bob, 2, &proof),
        None,
        "alice no longer owns it"
    );
    assert_eq!(MerkleTree::new(3).root(), MerkleTree::new(3).root());
    assert_eq!(
        cnft::root_from_proof(&cnft::EMPTY, 0, &MerkleTree::new(2).proof(0)),
        MerkleTree::new(2).root()
    );
}
