//! The on-chain track: the programs built by `./build-sbf.sh`, in LiteSVM.

use crate::WHICH;
use crate::sut::ex02_metadata::{self as metadata, CreateArgs, Metadata};
use crate::sut::ex03_vending::{self as vending, Config};
use crate::sut::ex04_nft_staking as nft_staking;
use solana_keypair::Keypair;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::instruction as system_instruction;
use solsim::ata::{create_associated_token_account, get_associated_token_address};
use solsim::sbf::Vm;
use spl_token_interface::instruction as token_ix;

fn vm() -> Vm {
    let mut vm = Vm::new();
    vm.load(metadata::ID, WHICH, "metadata");
    vm.load(vending::ID, WHICH, "vending");
    vm.load(nft_staking::ID, WHICH, "nft-staking");
    vm
}

fn balance(vm: &Vm, account: &Pubkey) -> u64 {
    spl_token_interface::state::Account::unpack(&vm.data(account)).map_or(0, |a| a.amount)
}

/// An NFT minted with plain transactions (the same steps as `mint_nft`).
fn mint_nft(vm: &mut Vm, creator: &Keypair, owner: &Pubkey, collection: Option<Pubkey>) -> Pubkey {
    let mint = Keypair::new();
    let (c, m) = (creator.pubkey(), mint.pubkey());
    let space = spl_token_interface::state::Mint::LEN;
    let args = CreateArgs {
        name: "On-chain".into(),
        symbol: "VM".into(),
        uri: "u".into(),
        seller_fee_basis_points: 0,
        creators: vec![(c, 100)],
        collection,
        is_mutable: true,
    };
    let ixs = [
        system_instruction::create_account(
            &c,
            &m,
            vm.minimum_balance(space),
            space as u64,
            &spl_token_interface::ID,
        ),
        token_ix::initialize_mint2(&spl_token_interface::ID, &m, &c, None, 0).unwrap(),
        create_associated_token_account(&c, owner, &m),
        token_ix::mint_to(
            &spl_token_interface::ID,
            &m,
            &get_associated_token_address(owner, &m),
            &c,
            &[],
            1,
        )
        .unwrap(),
        metadata::create(&c, &m, &c, args),
        token_ix::set_authority(
            &spl_token_interface::ID,
            &m,
            None,
            token_ix::AuthorityType::MintTokens,
            &c,
            &[],
        )
        .unwrap(),
    ];
    vm.process(&ixs, &[creator, &mint]).unwrap();
    m
}

#[test]
fn sbf_metadata_and_collections() {
    let mut vm = vm();
    let artist = vm.wallet(SOL);
    let a = artist.pubkey();
    let collection = mint_nft(&mut vm, &artist, &a, None);
    let nft = mint_nft(&mut vm, &artist, &a, Some(collection));
    vm.process(
        &[metadata::verify_collection(&a, &nft, &collection)],
        &[&artist],
    )
    .unwrap();
    let meta = Metadata::read(&vm.data(&metadata::metadata_address(&nft))).unwrap();
    assert!(meta.collection.unwrap().verified);
    assert!(meta.creators[0].verified);
}

#[test]
fn sbf_vending_machine() {
    let mut vm = vm();
    let (creator, treasury) = (vm.wallet(SOL), Pubkey::new_unique());
    let config = Config {
        price: SOL / 10,
        items_available: 2,
        go_live: 0,
        per_wallet_limit: 1,
        name_prefix: "VM".into(),
        base_uri: "https://e.com".into(),
    };
    vm.process(
        &[vending::initialize(&creator.pubkey(), 1, &treasury, config)],
        &[&creator],
    )
    .unwrap();
    let machine = vending::machine_address(&creator.pubkey(), 1);
    let buyer = vm.wallet(SOL);
    let new_mint = Keypair::new();
    let meta = vm
        .process(
            &[vending::mint(
                &buyer.pubkey(),
                &machine,
                &treasury,
                &new_mint.pubkey(),
            )],
            &[&buyer, &new_mint],
        )
        .unwrap();
    println!(
        "a vending machine mint used {} compute units",
        meta.compute_units
    );
    assert_eq!(vm.lamports(&treasury), SOL / 10);
    assert_eq!(
        balance(
            &vm,
            &get_associated_token_address(&buyer.pubkey(), &new_mint.pubkey())
        ),
        1
    );
    let nft_meta =
        Metadata::read(&vm.data(&metadata::metadata_address(&new_mint.pubkey()))).unwrap();
    assert_eq!(nft_meta.name, "VM #1");
}

#[test]
fn sbf_nft_staking() {
    let mut vm = vm();
    let artist = vm.wallet(SOL);
    let holder = vm.wallet(SOL);
    let (a, h) = (artist.pubkey(), holder.pubkey());
    let collection = mint_nft(&mut vm, &artist, &a, None);
    let nft = mint_nft(&mut vm, &artist, &h, Some(collection));
    vm.process(
        &[metadata::verify_collection(&a, &nft, &collection)],
        &[&artist],
    )
    .unwrap();
    // the reward mint, with the farm as authority
    let reward = Keypair::new();
    let space = spl_token_interface::state::Mint::LEN;
    let farm = nft_staking::farm_address(&collection);
    vm.process(
        &[
            system_instruction::create_account(
                &a,
                &reward.pubkey(),
                vm.minimum_balance(space),
                space as u64,
                &spl_token_interface::ID,
            ),
            token_ix::initialize_mint2(&spl_token_interface::ID, &reward.pubkey(), &farm, None, 0)
                .unwrap(),
            nft_staking::init_farm(&a, &collection, &reward.pubkey(), 5),
            create_associated_token_account(&a, &h, &reward.pubkey()),
        ],
        &[&artist, &reward],
    )
    .unwrap();
    let holder_nft = get_associated_token_address(&h, &nft);
    let holder_rewards = get_associated_token_address(&h, &reward.pubkey());
    vm.process(
        &[nft_staking::stake(&h, &collection, &nft, &holder_nft)],
        &[&holder],
    )
    .unwrap();
    let mut clock: solana_program::clock::Clock = vm.svm.get_sysvar();
    clock.unix_timestamp += 10;
    vm.svm.set_sysvar(&clock);
    vm.process(
        &[nft_staking::unstake(
            &h,
            &collection,
            &nft,
            &holder_nft,
            &reward.pubkey(),
            &holder_rewards,
        )],
        &[&holder],
    )
    .unwrap();
    assert_eq!(
        (balance(&vm, &holder_nft), balance(&vm, &holder_rewards)),
        (1, 50)
    );
}
