// Reference solution for 30-defi-protocols.
//
//     cargo run -p m30-defi-protocols-solution -- <1-5|bonus|all>

use borsh::BorshDeserialize;
use m30_defi_protocols_solution::ex03_lending_math::{self as lmath, RateModel, WAD};
use m30_defi_protocols_solution::ex05_lending::{
    self as lending, Market, MarketKeys, MarketParams, Obligation,
};
use m30_defi_protocols_solution::{
    bonus_sandwich as sandwich, ex01_amm_math as amm_math, ex02_amm as amm, ex04_oracle as oracle,
};
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solsim::{Sim, token};

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(amm::ID, amm::process);
    sim.add_program(oracle::ID, oracle::process);
    sim.add_program(lending::ID, lending::process);
    sim
}

fn ex1() {
    println!("--- 1: constant-product maths (pool 1,000,000 / 1,000,000, fee 0.3%)");
    for amount in [1_000, 10_000, 100_000, 500_000] {
        let out = amm_math::swap_out(amount, 1_000_000, 1_000_000, 30).unwrap();
        let impact = amm_math::price_impact_bps(amount, 1_000_000, 1_000_000, 30).unwrap();
        println!(
            "swap {amount:>7} A -> {out:>7} B   price impact {:.2}%",
            impact as f64 / 100.0
        );
    }
    let (user, locked) = amm_math::initial_shares(4_000_000, 1_000_000).unwrap();
    println!("first deposit 4,000,000 A + 1,000,000 B -> {user} shares (+{locked} locked forever)");
}

fn ex2() {
    println!("--- 2: the AMM program");
    let mut sim = sim();
    let issuer = sim.funded_wallet(10 * SOL);
    let (mint_a, mint_b) = amm::sorted(
        token::create_mint(&mut sim, &issuer, &issuer, 6),
        token::create_mint(&mut sim, &issuer, &issuer, 6),
    );
    sim.process_ix(amm::init_pool(&issuer, &mint_a, &mint_b, 30), &[issuer])
        .unwrap();
    let pool = amm::pool_address(&mint_a, &mint_b);
    let [acct_a, acct_b, lp] = [mint_a, mint_b, amm::lp_mint_address(&pool)]
        .map(|m| token::create_ata(&mut sim, &issuer, &issuer, &m));
    token::mint_to(&mut sim, &mint_a, &issuer, &acct_a, 2_000_000);
    token::mint_to(&mut sim, &mint_b, &issuer, &acct_b, 1_000_000);
    sim.process_ix(
        amm::add_liquidity(
            &issuer, &pool, &acct_a, &acct_b, &lp, 1_000_000, 1_000_000, 0,
        ),
        &[issuer],
    )
    .unwrap();
    println!("LP shares: {}", token::balance(&sim, &lp));
    let quote = amm_math::swap_out(100_000, 1_000_000, 1_000_000, 30).unwrap();
    let err = sim
        .process_ix(
            amm::swap(&issuer, &pool, &acct_a, &acct_b, 100_000, quote + 1, true),
            &[issuer],
        )
        .unwrap_err();
    println!(
        "swap 100,000 A asking for {} B: {:?} (SlippageExceeded = 2)",
        quote + 1,
        err.error
    );
    sim.process_ix(
        amm::swap(&issuer, &pool, &acct_a, &acct_b, 100_000, quote, true),
        &[issuer],
    )
    .unwrap();
    let state = amm::Pool::deserialize(&mut sim.data(&pool)).unwrap();
    println!(
        "swapped for {quote} B; reserves now {} / {}",
        state.reserve_a, state.reserve_b
    );
}

fn ex3() {
    println!("--- 3: lending maths");
    let model = RateModel {
        base_bps: 200,
        slope1_bps: 1_000,
        slope2_bps: 10_000,
        kink_bps: 8_000,
    };
    for u in [0, 4_000, 8_000, 9_000, 10_000] {
        println!(
            "utilization {:>3}% -> borrow rate {:>5.2}%",
            u / 100,
            model.borrow_rate_bps(u) as f64 / 100.0
        );
    }
    let index = lmath::accrue_index(WAD, 1_200, lmath::SECONDS_PER_YEAR as u64).unwrap();
    println!("a year at 12%: index {:.4}", index as f64 / WAD as f64);
    let hf = lmath::health_factor(800_000_000, 8_000, 700_000_000);
    println!(
        "$800 collateral, 80% threshold, $700 debt -> health {:.3}",
        hf as f64 / WAD as f64
    );
}

fn ex45() {
    println!("--- 4, 5: an oracle and a lending market");
    let mut sim = sim();
    let issuer = sim.funded_wallet(10 * SOL);
    let sol_mint = token::create_mint(&mut sim, &issuer, &issuer, 9);
    let usd_mint = token::create_mint(&mut sim, &issuer, &issuer, 6);
    let publisher = sim.funded_wallet(SOL);
    let publish = |sim: &mut Sim, sol_price: i64| {
        let ixs = [
            oracle::publish(&publisher, 1, sol_price, 10_000),
            oracle::publish(&publisher, 2, 1_000_000, 100),
        ];
        sim.process(&ixs, &[publisher]).unwrap();
    };
    sim.process(
        &[
            oracle::create(&publisher, 1, -6),
            oracle::create(&publisher, 2, -6),
        ],
        &[publisher],
    )
    .unwrap();
    publish(&mut sim, 100_000_000);
    let keys = MarketKeys::new(
        &sol_mint,
        &usd_mint,
        &oracle::feed_address(&publisher, 1),
        &oracle::feed_address(&publisher, 2),
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
    sim.process_ix(
        lending::init_market(&issuer, &sol_mint, &usd_mint, &keys, params),
        &[issuer],
    )
    .unwrap();
    let liquidity = token::create_ata(&mut sim, &issuer, &issuer, &usd_mint);
    token::mint_to(&mut sim, &usd_mint, &issuer, &liquidity, 10_000_000_000);
    sim.process_ix(
        lending::supply(&issuer, &keys, &liquidity, 10_000_000_000),
        &[issuer],
    )
    .unwrap();

    let user = sim.funded_wallet(SOL);
    let [user_sol, user_usd] =
        [sol_mint, usd_mint].map(|m| token::create_ata(&mut sim, &issuer, &user, &m));
    token::mint_to(&mut sim, &sol_mint, &issuer, &user_sol, 10_000_000_000);
    sim.process_ix(
        lending::deposit(&user, &keys, &user_sol, 10_000_000_000),
        &[user],
    )
    .unwrap();
    println!("deposited 10 SOL at $100");
    let err = sim
        .process_ix(
            lending::borrow(&user, &keys, &user_usd, 800_000_000),
            &[user],
        )
        .unwrap_err();
    println!("borrow $800: {:?} (ExceedsLtv = 1)", err.error);
    sim.process_ix(
        lending::borrow(&user, &keys, &user_usd, 700_000_000),
        &[user],
    )
    .unwrap();
    println!("borrowed $700");
    sim.advance_time(120);
    let err = sim
        .process_ix(lending::borrow(&user, &keys, &user_usd, 1), &[user])
        .unwrap_err();
    println!(
        "two minutes without a price update: {:?} (oracle StalePrice = 1)",
        err.error
    );

    publish(&mut sim, 80_000_000);
    println!("SOL drops to $80");
    let liquidator = sim.funded_wallet(SOL);
    let [l_sol, l_usd] =
        [sol_mint, usd_mint].map(|m| token::create_ata(&mut sim, &issuer, &liquidator, &m));
    token::mint_to(&mut sim, &usd_mint, &issuer, &l_usd, 1_000_000_000);
    sim.process_ix(
        lending::liquidate(&liquidator, &user, &keys, &l_usd, &l_sol, u64::MAX),
        &[liquidator],
    )
    .unwrap();
    println!(
        "liquidated: paid ${:.2}, received {:.4} SOL",
        (1_000_000_000 - token::balance(&sim, &l_usd)) as f64 / 1e6,
        token::balance(&sim, &l_sol) as f64 / 1e9
    );
    let m = Market::deserialize(&mut sim.data(&keys.market)).unwrap();
    let o =
        Obligation::deserialize(&mut sim.data(&lending::obligation_address(&keys.market, &user)))
            .unwrap();
    println!(
        "the borrower now owes ${:.2} against {:.4} SOL",
        lmath::current_debt(o.debt_scaled, m.borrow_index).unwrap() as f64 / 1e6,
        o.collateral as f64 / 1e9
    );
}

fn bonus() {
    println!("--- bonus: a sandwich attack on a 50,000,000 swap into a 1e9/1e9 pool");
    let (ra, rb, fee, victim) = (1_000_000_000, 1_000_000_000, 30, 50_000_000);
    let sizes: Vec<u64> = (1..=50).map(|i| i * 10_000_000).collect();
    for slippage_bps in [1_000, 300, 100, 10] {
        let min_out = sandwich::min_out_with_slippage(victim, ra, rb, fee, slippage_bps);
        match sandwich::best_attack(ra, rb, fee, victim, min_out, &sizes) {
            Some((size, s)) => println!(
                "slippage {:>5.1}%: attacker front-runs {size}, profits {}, victim gets {}",
                slippage_bps as f64 / 100.0,
                s.profit,
                s.victim_out
            ),
            None => println!(
                "slippage {:>5.1}%: no profitable sandwich",
                slippage_bps as f64 / 100.0
            ),
        }
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("1") => ex1(),
        Some("2") => ex2(),
        Some("3") => ex3(),
        Some("4") | Some("5") => ex45(),
        Some("bonus") => bonus(),
        Some("all") => {
            ex1();
            ex2();
            ex3();
            ex45();
            bonus();
        }
        _ => println!(
            "30-defi-protocols -- reference solution\n\n  cargo run -p m30-defi-protocols-solution -- <1-5|bonus|all>"
        ),
    }
}
