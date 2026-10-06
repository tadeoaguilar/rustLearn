//! The on-chain track: the programs built by `./build-sbf.sh`, in LiteSVM.

use crate::WHICH;
use crate::sut::ex05_lending::{self as lending, MarketKeys, MarketParams};
use crate::sut::{ex01_amm_math as amm_math, ex02_amm as amm, ex04_oracle as oracle};
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
    vm.load(amm::ID, WHICH, "amm");
    vm.load(oracle::ID, WHICH, "oracle");
    vm.load(lending::ID, WHICH, "lending");
    vm
}

fn create_mint(vm: &mut Vm, payer: &Keypair, decimals: u8) -> Pubkey {
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
        token_ix::initialize_mint2(
            &spl_token_interface::ID,
            &mint.pubkey(),
            &payer.pubkey(),
            None,
            decimals,
        )
        .unwrap(),
    ];
    vm.process(&ixs, &[payer, &mint]).unwrap();
    mint.pubkey()
}

/// `owner`'s ATA for `mint`, created and funded with `amount` (minted by `issuer`).
fn funded_ata(vm: &mut Vm, issuer: &Keypair, owner: &Pubkey, mint: &Pubkey, amount: u64) -> Pubkey {
    let ata = get_associated_token_address(owner, mint);
    let mut ixs = vec![create_associated_token_account(
        &issuer.pubkey(),
        owner,
        mint,
    )];
    if amount > 0 {
        ixs.push(
            token_ix::mint_to(
                &spl_token_interface::ID,
                mint,
                &ata,
                &issuer.pubkey(),
                &[],
                amount,
            )
            .unwrap(),
        );
    }
    vm.process(&ixs, &[issuer]).unwrap();
    ata
}

fn balance(vm: &Vm, account: &Pubkey) -> u64 {
    spl_token_interface::state::Account::unpack(&vm.data(account)).map_or(0, |a| a.amount)
}

#[test]
fn sbf_amm_round_trip() {
    let mut vm = vm();
    let issuer = vm.wallet(10 * SOL);
    let i = issuer.pubkey();
    let (mint_a, mint_b) = amm::sorted(
        create_mint(&mut vm, &issuer, 6),
        create_mint(&mut vm, &issuer, 6),
    );
    vm.process(&[amm::init_pool(&i, &mint_a, &mint_b, 30)], &[&issuer])
        .unwrap();
    let pool = amm::pool_address(&mint_a, &mint_b);
    let a = funded_ata(&mut vm, &issuer, &i, &mint_a, 2_000_000);
    let b = funded_ata(&mut vm, &issuer, &i, &mint_b, 1_000_000);
    let lp = funded_ata(&mut vm, &issuer, &i, &amm::lp_mint_address(&pool), 0);
    vm.process(
        &[amm::add_liquidity(
            &i, &pool, &a, &b, &lp, 1_000_000, 1_000_000, 0,
        )],
        &[&issuer],
    )
    .unwrap();
    let quote = amm_math::swap_out(100_000, 1_000_000, 1_000_000, 30).unwrap();
    let meta = vm
        .process(
            &[amm::swap(&i, &pool, &a, &b, 100_000, quote, true)],
            &[&issuer],
        )
        .unwrap();
    println!("a swap used {} compute units", meta.compute_units);
    assert_eq!(balance(&vm, &b), quote);
    vm.process(
        &[amm::remove_liquidity(&i, &pool, &a, &b, &lp, 999_000, 0, 0)],
        &[&issuer],
    )
    .unwrap();
    assert_eq!(balance(&vm, &lp), 0);
}

#[test]
fn sbf_lending_liquidation() {
    let mut vm = vm();
    let issuer = vm.wallet(10 * SOL);
    let i = issuer.pubkey();
    let sol_mint = create_mint(&mut vm, &issuer, 9);
    let usd_mint = create_mint(&mut vm, &issuer, 6);
    vm.process(
        &[oracle::create(&i, 1, -6), oracle::create(&i, 2, -6)],
        &[&issuer],
    )
    .unwrap();
    let publish = |vm: &mut Vm, sol: i64| {
        vm.process(
            &[
                oracle::publish(&i, 1, sol, 10_000),
                oracle::publish(&i, 2, 1_000_000, 100),
            ],
            &[&issuer],
        )
        .unwrap();
    };
    publish(&mut vm, 100_000_000);
    let keys = MarketKeys::new(
        &sol_mint,
        &usd_mint,
        &oracle::feed_address(&i, 1),
        &oracle::feed_address(&i, 2),
    );
    let params = MarketParams {
        ltv_bps: 7_500,
        liquidation_threshold_bps: 8_000,
        liquidation_bonus_bps: 500,
        close_factor_bps: 5_000,
        base_rate_bps: 200,
        slope1_bps: 1_000,
        slope2_bps: 10_000,
        kink_bps: 8_000,
    };
    vm.process(
        &[lending::init_market(
            &i, &sol_mint, &usd_mint, &keys, params,
        )],
        &[&issuer],
    )
    .unwrap();
    let liquidity = funded_ata(&mut vm, &issuer, &i, &usd_mint, 10_000_000_000);
    vm.process(
        &[lending::supply(&i, &keys, &liquidity, 10_000_000_000)],
        &[&issuer],
    )
    .unwrap();

    let user = vm.wallet(SOL);
    let u = user.pubkey();
    let user_sol = funded_ata(&mut vm, &issuer, &u, &sol_mint, 10_000_000_000);
    let user_usd = funded_ata(&mut vm, &issuer, &u, &usd_mint, 0);
    vm.process(
        &[
            lending::deposit(&u, &keys, &user_sol, 10_000_000_000),
            lending::borrow(&u, &keys, &user_usd, 700_000_000),
        ],
        &[&user],
    )
    .unwrap();
    assert_eq!(balance(&vm, &user_usd), 700_000_000);

    publish(&mut vm, 80_000_000);
    let liquidator = vm.wallet(SOL);
    let l = liquidator.pubkey();
    let l_usd = funded_ata(&mut vm, &issuer, &l, &usd_mint, 1_000_000_000);
    let l_sol = funded_ata(&mut vm, &issuer, &l, &sol_mint, 0);
    vm.process(
        &[lending::liquidate(&l, &u, &keys, &l_usd, &l_sol, u64::MAX)],
        &[&liquidator],
    )
    .unwrap();
    assert_eq!(balance(&vm, &l_sol), 4_593_750_000);
}
