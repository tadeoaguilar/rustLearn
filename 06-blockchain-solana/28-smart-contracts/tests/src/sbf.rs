//! The on-chain track: the programs built by `./build-sbf.sh`, in LiteSVM.

use crate::WHICH;
use crate::sut::ex04_security::{self as bank, secure_id};
use crate::sut::{ex01_multisig as ms, ex02_escrow as escrow, ex03_staking as staking};
use solana_keypair::Keypair;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::instruction as system_instruction;
use solsim::sbf::Vm;

fn vm() -> Vm {
    let mut vm = Vm::new();
    vm.load(ms::ID, WHICH, "multisig");
    vm.load(escrow::ID, WHICH, "escrow");
    vm.load(staking::ID, WHICH, "staking");
    vm.load(secure_id::ID, WHICH, "bank-secure");
    vm
}

fn create_mint(vm: &mut Vm, payer: &Keypair, authority: &Pubkey) -> Pubkey {
    let mint = Keypair::new();
    let space = spl_token_interface::state::Mint::LEN;
    let ixs = [
        system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            vm.minimum_balance(space),
            space as u64,
            &spl_token_interface::ID,
        ),
        spl_token_interface::instruction::initialize_mint2(
            &spl_token_interface::ID,
            &mint.pubkey(),
            authority,
            None,
            0,
        )
        .unwrap(),
    ];
    vm.process(&ixs, &[payer, &mint]).unwrap();
    mint.pubkey()
}

fn create_token_account(vm: &mut Vm, payer: &Keypair, mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    let account = Keypair::new();
    let space = spl_token_interface::state::Account::LEN;
    let ixs = [
        system_instruction::create_account(
            &payer.pubkey(),
            &account.pubkey(),
            vm.minimum_balance(space),
            space as u64,
            &spl_token_interface::ID,
        ),
        spl_token_interface::instruction::initialize_account3(
            &spl_token_interface::ID,
            &account.pubkey(),
            mint,
            owner,
        )
        .unwrap(),
    ];
    vm.process(&ixs, &[payer, &account]).unwrap();
    account.pubkey()
}

fn mint_to(vm: &mut Vm, mint: &Pubkey, authority: &Keypair, to: &Pubkey, amount: u64) {
    let ix = spl_token_interface::instruction::mint_to(
        &spl_token_interface::ID,
        mint,
        to,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    vm.process(&[ix], &[authority]).unwrap();
}

fn balance(vm: &Vm, account: &Pubkey) -> u64 {
    spl_token_interface::state::Account::unpack(&vm.data(account)).map_or(0, |a| a.amount)
}

#[test]
fn sbf_multisig_executes_an_approved_transfer() {
    let mut vm = vm();
    let owners = [vm.wallet(SOL), vm.wallet(SOL)];
    let keys = owners.each_ref().map(|o| o.pubkey());
    vm.process(&[ms::create(&keys[0], 1, &keys, 2)], &[&owners[0]])
        .unwrap();
    let multisig = ms::multisig_address(&keys[0], 1);
    let vault = ms::vault_address(&multisig);
    vm.svm.airdrop(&vault, 3 * SOL).unwrap();
    let payee = Pubkey::new_unique();
    let inner = system_instruction::transfer(&vault, &payee, SOL);
    vm.process(
        &[ms::propose(&keys[0], &multisig, 0, &inner)],
        &[&owners[0]],
    )
    .unwrap();
    assert!(
        vm.process(
            &[ms::execute(&keys[0], &multisig, 0, &inner)],
            &[&owners[0]]
        )
        .is_err()
    );
    vm.process(&[ms::approve(&keys[1], &multisig, 0)], &[&owners[1]])
        .unwrap();
    vm.process(
        &[ms::execute(&keys[1], &multisig, 0, &inner)],
        &[&owners[1]],
    )
    .unwrap();
    assert_eq!(vm.lamports(&payee), SOL);
}

#[test]
fn sbf_escrow_swap() {
    let mut vm = vm();
    let (maker, taker, issuer) = (vm.wallet(SOL), vm.wallet(SOL), vm.wallet(SOL));
    let mint_a = create_mint(&mut vm, &issuer, &issuer.pubkey());
    let mint_b = create_mint(&mut vm, &issuer, &issuer.pubkey());
    let maker_a = create_token_account(&mut vm, &issuer, &mint_a, &maker.pubkey());
    let maker_b = create_token_account(&mut vm, &issuer, &mint_b, &maker.pubkey());
    let taker_a = create_token_account(&mut vm, &issuer, &mint_a, &taker.pubkey());
    let taker_b = create_token_account(&mut vm, &issuer, &mint_b, &taker.pubkey());
    mint_to(&mut vm, &mint_a, &issuer, &maker_a, 10);
    mint_to(&mut vm, &mint_b, &issuer, &taker_b, 20);
    let make = escrow::make(
        &maker.pubkey(),
        1,
        &mint_a,
        &mint_b,
        &maker_a,
        &maker_b,
        10,
        20,
    );
    vm.process(&[make], &[&maker]).unwrap();
    let escrow_key = escrow::escrow_address(&maker.pubkey(), 1);
    let take = escrow::take(
        &taker.pubkey(),
        &maker.pubkey(),
        &escrow_key,
        &taker_a,
        &taker_b,
        &maker_b,
    );
    let meta = vm.process(&[take], &[&taker]).unwrap();
    println!("take used {} compute units", meta.compute_units);
    assert_eq!((balance(&vm, &maker_b), balance(&vm, &taker_a)), (20, 10));
    assert!(vm.account(&escrow_key).is_none());
}

#[test]
fn sbf_staking_rewards() {
    let mut vm = vm();
    let admin = vm.wallet(SOL);
    let stake_mint = create_mint(&mut vm, &admin, &admin.pubkey());
    let pool = staking::pool_address(&stake_mint);
    let reward_mint = create_mint(&mut vm, &admin, &pool);
    vm.process(
        &[staking::init_pool(
            &admin.pubkey(),
            &stake_mint,
            &reward_mint,
            10,
        )],
        &[&admin],
    )
    .unwrap();
    let user = vm.wallet(SOL);
    let tokens = create_token_account(&mut vm, &admin, &stake_mint, &user.pubkey());
    let rewards = create_token_account(&mut vm, &admin, &reward_mint, &user.pubkey());
    mint_to(&mut vm, &stake_mint, &admin, &tokens, 50);
    vm.process(
        &[staking::stake(&user.pubkey(), &stake_mint, &tokens, 50)],
        &[&user],
    )
    .unwrap();
    let mut clock: solana_program::clock::Clock = vm.svm.get_sysvar();
    clock.unix_timestamp += 30;
    vm.svm.set_sysvar(&clock);
    vm.process(
        &[staking::claim(
            &user.pubkey(),
            &stake_mint,
            &reward_mint,
            &rewards,
        )],
        &[&user],
    )
    .unwrap();
    assert_eq!(balance(&vm, &rewards), 300);
}

#[test]
fn sbf_secure_bank() {
    let mut vm = vm();
    let program = secure_id::ID;
    let admin = vm.wallet(SOL);
    vm.process(&[bank::init(&program, &admin.pubkey())], &[&admin])
        .unwrap();
    let user = vm.wallet(5 * SOL);
    let u = user.pubkey();
    vm.process(
        &[
            bank::open(&program, &u),
            bank::deposit(&program, &u, 2 * SOL),
        ],
        &[&user],
    )
    .unwrap();
    vm.process(&[bank::withdraw(&program, &u, SOL)], &[&user])
        .unwrap();
    assert_eq!(vm.lamports(&bank::vault_address(&program)), SOL);
    // bug 6 on the VM: transferring to yourself is refused
    assert!(
        vm.process(&[bank::transfer(&program, &u, &u, SOL)], &[&user])
            .is_err()
    );
}
