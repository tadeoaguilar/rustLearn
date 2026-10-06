use crate::sut::bonus_sandwich as sandwich;
use crate::sut::ex01_amm_math::{self as amm_math, MINIMUM_LIQUIDITY};
use crate::sut::ex02_amm::{self as amm, AmmError, Pool};
use crate::sut::ex03_lending_math::{self as lmath, Liquidation, RateModel, WAD};
use crate::sut::ex04_oracle::{self as oracle, OracleError, Price, PriceFeed};
use crate::sut::ex05_lending::{
    self as lending, LendingError, Market, MarketKeys, MarketParams, Obligation,
};
use borsh::BorshDeserialize;
use proptest::prelude::*;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::pubkey::Pubkey;
use solsim::{Account, Sim, TxError, TxMeta, token};

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(amm::ID, amm::process);
    sim.add_program(oracle::ID, oracle::process);
    sim.add_program(lending::ID, lending::process);
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

// ---------------------------------------------------------------- Exercise 1

#[test]
fn ex1_isqrt() {
    for (n, r) in [
        (0u128, 0u128),
        (1, 1),
        (2, 1),
        (3, 1),
        (4, 2),
        (15, 3),
        (16, 4),
        (17, 4),
        (1 << 64, 1 << 32),
    ] {
        assert_eq!(amm_math::isqrt(n), r, "isqrt({n})");
    }
    let big = u64::MAX as u128 * u64::MAX as u128;
    assert_eq!(amm_math::isqrt(big), u64::MAX as u128);
    assert_eq!(amm_math::isqrt(u128::MAX), u64::MAX as u128);
}

#[test]
fn ex1_swap_out() {
    assert_eq!(amm_math::swap_out(100, 1000, 1000, 0), Some(90)); // 100 * 1000 / 1100 = 90.9
    assert_eq!(
        amm_math::swap_out(1_000, 1_000_000, 1_000_000, 30),
        Some(996)
    );
    assert_eq!(amm_math::swap_out(0, 1000, 1000, 30), Some(0));
    assert_eq!(amm_math::swap_out(5, 0, 1000, 30), None);
    assert_eq!(amm_math::swap_out(5, 1000, 0, 30), None);
    assert_eq!(amm_math::swap_out(5, 1000, 1000, 10_001), None);
    // can never drain the pool
    assert!(amm_math::swap_out(u64::MAX, 1, 1_000_000, 0).unwrap() < 1_000_000);
}

#[test]
fn ex1_shares() {
    assert_eq!(
        amm_math::initial_shares(100_000_000, 100),
        Some((100_000 - MINIMUM_LIQUIDITY, MINIMUM_LIQUIDITY))
    );
    assert_eq!(
        amm_math::initial_shares(1000, 1000),
        None,
        "exactly the locked amount leaves nothing"
    );
    assert_eq!(amm_math::initial_shares(10, 10), None);
    // 10% more of each -> 10% more shares; amounts rounded up
    assert_eq!(
        amm_math::deposit(100, 200, 1000, 2000, 500),
        Some((50, 100, 200))
    );
    assert_eq!(
        amm_math::deposit(100, 100, 1000, 2000, 500),
        Some((25, 50, 100)),
        "limited by the scarcer side"
    );
    assert_eq!(
        amm_math::deposit(10, 10, 3, 3, 1),
        Some((3, 9, 9)),
        "each share costs 3 of each"
    );
    assert_eq!(
        amm_math::deposit(1, 1, 1000, 1000, 10),
        None,
        "would round to 0 shares"
    );
    let (shares, a, b) = amm_math::deposit(7, 7, 10, 10, 3).unwrap();
    assert_eq!((shares, a, b), (2, 7, 7), "2 shares cost 6.67 -> 7 of each");
    assert_eq!(amm_math::withdraw(50, 1000, 2000, 500), Some((100, 200)));
    assert_eq!(amm_math::withdraw(1, 10, 10, 3), Some((3, 3)));
    assert_eq!(amm_math::withdraw(501, 1000, 2000, 500), None);
}

#[test]
fn ex1_price_impact_grows_with_size() {
    let small = amm_math::price_impact_bps(1_000, 1_000_000, 1_000_000, 0).unwrap();
    let large = amm_math::price_impact_bps(100_000, 1_000_000, 1_000_000, 0).unwrap();
    assert!(small < 20 && large > 800, "{small} {large}");
    assert!((amm_math::spot_price(1_000, 3_000) - 3.0).abs() < 1e-9);
}

proptest! {
    #[test]
    fn ex1_isqrt_is_the_floor_root(n in any::<u128>()) {
        let r = amm_math::isqrt(n);
        prop_assert!(r * r <= n);
        prop_assert!((r + 1).checked_mul(r + 1).is_none_or(|sq| sq > n));
    }

    #[test]
    fn ex1_swaps_never_shrink_k(ra in 1_000u64..1_000_000_000_000, rb in 1_000u64..1_000_000_000_000, amount in 1u64..1_000_000_000_000, fee in 0u64..100) {
        let out = amm_math::swap_out(amount, ra, rb, fee).unwrap();
        prop_assert!(out < rb);
        let k_before = ra as u128 * rb as u128;
        let k_after = (ra as u128 + amount as u128) * (rb - out) as u128;
        prop_assert!(k_after >= k_before);
    }

    #[test]
    fn ex1_deposit_then_withdraw_never_profits(ra in 1_000u64..1_000_000_000, rb in 1_000u64..1_000_000_000, supply in 1_000u64..1_000_000_000, a in 1u64..1_000_000_000, b in 1u64..1_000_000_000) {
        if let Some((shares, paid_a, paid_b)) = amm_math::deposit(a, b, ra, rb, supply) {
            prop_assert!(paid_a <= a && paid_b <= b);
            let (got_a, got_b) = amm_math::withdraw(shares, ra + paid_a, rb + paid_b, supply + shares).unwrap();
            prop_assert!(got_a <= paid_a && got_b <= paid_b);
        }
    }
}

// ---------------------------------------------------------------- Exercise 2

struct Dex {
    issuer: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    pool: Pubkey,
}

fn dex(sim: &mut Sim, fee_bps: u64) -> Dex {
    let issuer = sim.funded_wallet(10 * SOL);
    let (mint_a, mint_b) = amm::sorted(
        token::create_mint(sim, &issuer, &issuer, 6),
        token::create_mint(sim, &issuer, &issuer, 6),
    );
    sim.process_ix(
        amm::init_pool(&issuer, &mint_a, &mint_b, fee_bps),
        &[issuer],
    )
    .unwrap();
    Dex {
        issuer,
        mint_a,
        mint_b,
        pool: amm::pool_address(&mint_a, &mint_b),
    }
}

/// A trader with `a` and `b` tokens and an LP account: (wallet, A account, B account, LP account).
fn trader(sim: &mut Sim, d: &Dex, a: u64, b: u64) -> (Pubkey, Pubkey, Pubkey, Pubkey) {
    let wallet = sim.funded_wallet(SOL);
    let acct_a = token::create_ata(sim, &d.issuer, &wallet, &d.mint_a);
    let acct_b = token::create_ata(sim, &d.issuer, &wallet, &d.mint_b);
    let lp = token::create_ata(sim, &d.issuer, &wallet, &amm::lp_mint_address(&d.pool));
    if a > 0 {
        token::mint_to(sim, &d.mint_a, &d.issuer, &acct_a, a);
    }
    if b > 0 {
        token::mint_to(sim, &d.mint_b, &d.issuer, &acct_b, b);
    }
    (wallet, acct_a, acct_b, lp)
}

fn pool_state(sim: &Sim, pool: &Pubkey) -> Pool {
    Pool::deserialize(&mut sim.data(pool)).unwrap()
}

#[test]
fn ex2_init_pool() {
    let mut sim = sim();
    let d = dex(&mut sim, 30);
    let state = pool_state(&sim, &d.pool);
    assert_eq!(
        (
            state.mint_a,
            state.mint_b,
            state.fee_bps,
            state.total_shares
        ),
        (d.mint_a, d.mint_b, 30, 0)
    );
    let lp = token::mint_state(&sim, &amm::lp_mint_address(&d.pool)).unwrap();
    assert_eq!(lp.mint_authority, COption::Some(d.pool));
    let (vault_a, vault_b) = amm::vault_addresses(&d.pool);
    assert_eq!(
        token::token_account_state(&sim, &vault_a).unwrap().owner,
        d.pool
    );
    assert_eq!(
        token::token_account_state(&sim, &vault_b).unwrap().mint,
        d.mint_b
    );

    assert_error(
        sim.process_ix(
            amm::init_pool(&d.issuer, &d.mint_b, &d.mint_a, 30),
            &[d.issuer],
        ),
        AmmError::UnorderedMints,
    );
    let (x, y) = amm::sorted(
        token::create_mint(&mut sim, &d.issuer, &d.issuer, 0),
        token::create_mint(&mut sim, &d.issuer, &d.issuer, 0),
    );
    assert_error(
        sim.process_ix(amm::init_pool(&d.issuer, &x, &y, 1000), &[d.issuer]),
        AmmError::BadFee,
    );
}

#[test]
fn ex2_liquidity_in_and_out() {
    let mut sim = sim();
    let d = dex(&mut sim, 30);
    let (alice, a_a, a_b, a_lp) = trader(&mut sim, &d, 1_000_000, 4_000_000);
    sim.process_ix(
        amm::add_liquidity(&alice, &d.pool, &a_a, &a_b, &a_lp, 1_000_000, 4_000_000, 0),
        &[alice],
    )
    .unwrap();
    // sqrt(1e6 * 4e6) = 2e6 shares, 1000 locked
    assert_eq!(token::balance(&sim, &a_lp), 2_000_000 - MINIMUM_LIQUIDITY);
    let state = pool_state(&sim, &d.pool);
    assert_eq!(
        (state.reserve_a, state.reserve_b, state.total_shares),
        (1_000_000, 4_000_000, 2_000_000)
    );

    let (bob, b_a, b_b, b_lp) = trader(&mut sim, &d, 100_000, 1_000_000);
    let too_greedy = amm::add_liquidity(
        &bob, &d.pool, &b_a, &b_b, &b_lp, 100_000, 1_000_000, 200_001,
    );
    assert_error(
        sim.process_ix(too_greedy, &[bob]),
        AmmError::SlippageExceeded,
    );
    sim.process_ix(
        amm::add_liquidity(
            &bob, &d.pool, &b_a, &b_b, &b_lp, 100_000, 1_000_000, 200_000,
        ),
        &[bob],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &b_lp), 200_000);
    assert_eq!(
        (token::balance(&sim, &b_a), token::balance(&sim, &b_b)),
        (0, 600_000),
        "took only the pool's ratio"
    );

    let out_of_range = amm::remove_liquidity(&bob, &d.pool, &b_a, &b_b, &b_lp, 200_000, 100_001, 0);
    assert_error(
        sim.process_ix(out_of_range, &[bob]),
        AmmError::SlippageExceeded,
    );
    sim.process_ix(
        amm::remove_liquidity(&bob, &d.pool, &b_a, &b_b, &b_lp, 200_000, 100_000, 400_000),
        &[bob],
    )
    .unwrap();
    assert_eq!(
        (
            token::balance(&sim, &b_a),
            token::balance(&sim, &b_b),
            token::balance(&sim, &b_lp)
        ),
        (100_000, 1_000_000, 0)
    );
    assert!(
        sim.process_ix(
            amm::remove_liquidity(&bob, &d.pool, &b_a, &b_b, &b_lp, 1, 0, 0),
            &[bob]
        )
        .is_err(),
        "no shares left to burn"
    );
}

#[test]
fn ex2_swaps_follow_the_curve() {
    let mut sim = sim();
    let d = dex(&mut sim, 30);
    let (lp, l_a, l_b, l_lp) = trader(&mut sim, &d, 1_000_000_000, 1_000_000_000);
    sim.process_ix(
        amm::add_liquidity(
            &lp,
            &d.pool,
            &l_a,
            &l_b,
            &l_lp,
            1_000_000_000,
            1_000_000_000,
            0,
        ),
        &[lp],
    )
    .unwrap();
    let (t, t_a, t_b, _) = trader(&mut sim, &d, 10_000_000, 0);

    let expected = amm_math::swap_out(10_000_000, 1_000_000_000, 1_000_000_000, 30).unwrap();
    assert_error(
        sim.process_ix(
            amm::swap(&t, &d.pool, &t_a, &t_b, 10_000_000, expected + 1, true),
            &[t],
        ),
        AmmError::SlippageExceeded,
    );
    assert_eq!(token::balance(&sim, &t_a), 10_000_000, "nothing moved");
    sim.process_ix(
        amm::swap(&t, &d.pool, &t_a, &t_b, 10_000_000, expected, true),
        &[t],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &t_b), expected);
    let state = pool_state(&sim, &d.pool);
    assert_eq!(
        (state.reserve_a, state.reserve_b),
        (1_010_000_000, 1_000_000_000 - expected)
    );
    assert!(state.reserve_a as u128 * state.reserve_b as u128 > 1_000_000_000u128 * 1_000_000_000);

    // and back
    let back = amm_math::swap_out(expected, state.reserve_b, state.reserve_a, 30).unwrap();
    sim.process_ix(amm::swap(&t, &d.pool, &t_b, &t_a, expected, 0, false), &[t])
        .unwrap();
    assert_eq!(token::balance(&sim, &t_a), back);
    assert!(back < 10_000_000, "a round trip costs the fees");
}

#[test]
fn ex2_accounts_are_checked() {
    let mut sim = sim();
    let d = dex(&mut sim, 30);
    let (lp, l_a, l_b, l_lp) = trader(&mut sim, &d, 1_000_000, 1_000_000);
    sim.process_ix(
        amm::add_liquidity(&lp, &d.pool, &l_a, &l_b, &l_lp, 1_000_000, 1_000_000, 0),
        &[lp],
    )
    .unwrap();
    let (t, t_a, t_b, _) = trader(&mut sim, &d, 1_000, 0);
    // destination is an A account
    assert_error(
        sim.process_ix(amm::swap(&t, &d.pool, &t_a, &t_a, 1_000, 0, true), &[t]),
        AmmError::WrongMint,
    );
    // a "vault" that isn't the pool's
    let mut fake_vault = amm::swap(&t, &d.pool, &t_a, &t_b, 1_000, 0, true);
    fake_vault.accounts[3].pubkey = l_b;
    assert_error(sim.process_ix(fake_vault, &[t]), AmmError::WrongPoolAccount);
    assert_error(
        sim.process_ix(amm::swap(&t, &d.pool, &t_a, &t_b, 0, 0, true), &[t]),
        AmmError::ZeroAmount,
    );
}

#[test]
fn ex2_donations_do_not_move_the_price() {
    let mut sim = sim();
    let d = dex(&mut sim, 30);
    let (lp, l_a, l_b, l_lp) = trader(&mut sim, &d, 1_000_000, 1_000_000);
    sim.process_ix(
        amm::add_liquidity(&lp, &d.pool, &l_a, &l_b, &l_lp, 1_000_000, 1_000_000, 0),
        &[lp],
    )
    .unwrap();
    // someone sends 500k B straight into the vault
    let (_, vault_b) = amm::vault_addresses(&d.pool);
    token::mint_to(&mut sim, &d.mint_b, &d.issuer, &vault_b, 500_000);
    let (t, t_a, t_b, _) = trader(&mut sim, &d, 10_000, 0);
    sim.process_ix(amm::swap(&t, &d.pool, &t_a, &t_b, 10_000, 0, true), &[t])
        .unwrap();
    assert_eq!(
        token::balance(&sim, &t_b),
        amm_math::swap_out(10_000, 1_000_000, 1_000_000, 30).unwrap(),
        "priced on stored reserves"
    );
}

// ---------------------------------------------------------------- Exercise 3

const MODEL: RateModel = RateModel {
    base_bps: 200,
    slope1_bps: 1_000,
    slope2_bps: 10_000,
    kink_bps: 8_000,
};

#[test]
fn ex3_utilization_and_rates() {
    assert_eq!(lmath::utilization_bps(0, 0), 0);
    assert_eq!(lmath::utilization_bps(50, 200), 2_500);
    assert_eq!(lmath::utilization_bps(300, 200), 10_000, "capped");
    assert_eq!(MODEL.borrow_rate_bps(0), 200);
    assert_eq!(MODEL.borrow_rate_bps(4_000), 700);
    assert_eq!(MODEL.borrow_rate_bps(8_000), 1_200);
    assert_eq!(MODEL.borrow_rate_bps(9_000), 6_200);
    assert_eq!(MODEL.borrow_rate_bps(10_000), 11_200);
}

#[test]
fn ex3_index_and_debts() {
    let year = lmath::SECONDS_PER_YEAR as u64;
    assert_eq!(
        lmath::accrue_index(WAD, 1_000, year),
        Some(WAD + WAD / 10),
        "10% for a year"
    );
    assert_eq!(lmath::accrue_index(WAD, 1_000, 0), Some(WAD));
    let half = lmath::accrue_index(WAD, 1_000, year / 2).unwrap();
    let twice = lmath::accrue_index(half, 1_000, year / 2).unwrap();
    assert!(twice > WAD + WAD / 10, "accruing more often compounds");
    assert_eq!(lmath::accrue_index(u128::MAX / 2, 1_000, year), None);

    let index = WAD + WAD / 3; // 1.333...
    let scaled = lmath::scaled_debt(100, index);
    let back = lmath::current_debt(scaled, index).unwrap();
    assert!(back >= 100, "rounding never forgives debt");
    assert_eq!(
        lmath::current_debt(lmath::scaled_debt(1_000, WAD), WAD + WAD / 10),
        Some(1_100)
    );
}

#[test]
fn ex3_values_health_and_limits() {
    // 2.5 tokens with 9 decimals at $40 (micro-dollars) = $100
    assert_eq!(lmath::value(2_500_000_000, 40_000_000, 9), 100_000_000);
    assert_eq!(lmath::max_borrow_value(1_000, 7_500), 750);
    assert_eq!(lmath::health_factor(1_000, 8_000, 800), WAD);
    assert!(lmath::health_factor(1_000, 8_000, 801) < WAD);
    assert_eq!(lmath::health_factor(1_000, 8_000, 0), u128::MAX);
}

#[test]
fn ex3_liquidation_amounts() {
    // debt 700 USD (6 dec), collateral 10 SOL (9 dec) at $80, bonus 5%, close factor 50%
    let l = lmath::liquidate(
        700_000_000,
        10_000_000_000,
        u64::MAX,
        5_000,
        500,
        1_000_000,
        6,
        80_000_000,
        9,
    );
    assert_eq!(
        l,
        Liquidation {
            repay: 350_000_000,
            seize: 4_593_750_000
        }
    );
    // a smaller offer
    let l = lmath::liquidate(
        700_000_000,
        10_000_000_000,
        100_000_000,
        5_000,
        500,
        1_000_000,
        6,
        80_000_000,
        9,
    );
    assert_eq!(l.repay, 100_000_000);
    // not enough collateral to pay the bonus on half the debt: repay shrinks
    let l = lmath::liquidate(
        700_000_000,
        1_000_000_000,
        u64::MAX,
        5_000,
        500,
        1_000_000,
        6,
        80_000_000,
        9,
    );
    assert_eq!(l.seize, 1_000_000_000);
    assert_eq!(l.repay, 76_190_476, "$80 of collateral covers $76.19 + 5%");
}

// ---------------------------------------------------------------- Exercise 4

fn feed(sim: &mut Sim, id: u64, price: i64, conf: u64) -> (Pubkey, Pubkey) {
    let authority = sim.funded_wallet(SOL);
    sim.process(
        &[
            oracle::create(&authority, id, -6),
            oracle::publish(&authority, id, price, conf),
        ],
        &[authority],
    )
    .unwrap();
    (authority, oracle::feed_address(&authority, id))
}

fn feed_state(sim: &Sim, key: &Pubkey) -> PriceFeed {
    PriceFeed::deserialize(&mut sim.data(key)).unwrap()
}

#[test]
fn ex4_publish_prices() {
    let mut sim = sim();
    let (authority, key) = feed(&mut sim, 1, 150_000_000, 10_000);
    let state = feed_state(&sim, &key);
    assert_eq!(
        (state.price, state.conf, state.expo, state.publish_time),
        (150_000_000, 10_000, -6, sim.clock().unix_timestamp)
    );
    sim.advance_time(5);
    sim.process_ix(
        oracle::publish(&authority, 1, 151_000_000, 9_000),
        &[authority],
    )
    .unwrap();
    assert_eq!(
        feed_state(&sim, &key).publish_time,
        sim.clock().unix_timestamp
    );
    let mallory = sim.funded_wallet(SOL);
    let mut forged = oracle::publish(&authority, 1, 1, 0);
    forged.accounts[0].pubkey = mallory;
    assert_error(
        sim.process_ix(forged, &[mallory]),
        OracleError::NotTheAuthority,
    );
}

#[test]
fn ex4_validation() {
    let state = PriceFeed {
        authority: Pubkey::default(),
        id: 0,
        price: 100_000_000,
        conf: 1_000_000,
        expo: -6,
        publish_time: 1_000,
        bump: 0,
    };
    assert_eq!(
        oracle::validate(&state, 1_030, 60, 200),
        Ok(Price {
            price: 100_000_000,
            expo: -6
        })
    );
    assert_eq!(
        oracle::validate(&state, 1_061, 60, 200),
        Err(OracleError::StalePrice.into())
    );
    assert_eq!(
        oracle::validate(&state, 1_000, 60, 50),
        Err(OracleError::ConfidenceTooWide.into())
    );
    let negative = PriceFeed {
        price: -5,
        ..state.clone()
    };
    assert_eq!(
        oracle::validate(&negative, 1_000, 60, 200),
        Err(OracleError::NonPositivePrice.into())
    );
    assert_eq!(
        Price {
            price: 123_456_789,
            expo: -6
        }
        .scaled(6),
        123_456_789
    );
    assert_eq!(
        Price {
            price: 123_456_789,
            expo: -8
        }
        .scaled(6),
        1_234_567
    );
    assert_eq!(Price { price: 42, expo: 0 }.scaled(6), 42_000_000);
}

#[test]
fn ex4_lookalike_feeds_are_rejected() {
    // read_price on an AccountInfo: owned by another program -> NotAFeed
    let key = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let state = PriceFeed {
        authority: owner,
        id: 0,
        price: 1,
        conf: 0,
        expo: 0,
        publish_time: 0,
        bump: 0,
    };
    let mut data = borsh::to_vec(&state).unwrap();
    let mut lamports = 1;
    let info = solana_program::account_info::AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
    );
    assert_eq!(
        oracle::read_price(&info, 0, 60, 200),
        Err(OracleError::NotAFeed.into())
    );
}

// ---------------------------------------------------------------- Exercise 5

const PARAMS: MarketParams = MarketParams {
    ltv_bps: 7_500,
    liquidation_threshold_bps: 8_000,
    liquidation_bonus_bps: 500,
    close_factor_bps: 5_000,
    base_rate_bps: 200,
    slope1_bps: 1_000,
    slope2_bps: 10_000,
    kink_bps: 8_000,
};
const USD: u64 = 1_000_000; // 6 decimals
const ONE_SOL: u64 = 1_000_000_000; // 9 decimals

struct Bank {
    issuer: Pubkey,
    sol_mint: Pubkey,
    usd_mint: Pubkey,
    sol_feed_authority: Pubkey,
    usd_feed_authority: Pubkey,
    keys: MarketKeys,
}

/// A market lending USD against SOL at $100, with $1M of liquidity.
fn bank(sim: &mut Sim) -> Bank {
    let issuer = sim.funded_wallet(10 * SOL);
    let sol_mint = token::create_mint(sim, &issuer, &issuer, 9);
    let usd_mint = token::create_mint(sim, &issuer, &issuer, 6);
    let (sol_feed_authority, sol_feed) = feed(sim, 1, 100 * USD as i64, 10_000);
    let (usd_feed_authority, usd_feed) = feed(sim, 2, USD as i64, 100);
    let keys = MarketKeys::new(&sol_mint, &usd_mint, &sol_feed, &usd_feed);
    sim.process_ix(
        lending::init_market(&issuer, &sol_mint, &usd_mint, &keys, PARAMS),
        &[issuer],
    )
    .unwrap();
    let liquidity = token::create_ata(sim, &issuer, &issuer, &usd_mint);
    token::mint_to(sim, &usd_mint, &issuer, &liquidity, 1_000_000 * USD);
    sim.process_ix(
        lending::supply(&issuer, &keys, &liquidity, 1_000_000 * USD),
        &[issuer],
    )
    .unwrap();
    Bank {
        issuer,
        sol_mint,
        usd_mint,
        sol_feed_authority,
        usd_feed_authority,
        keys,
    }
}

/// A borrower with `sol` collateral deposited; (wallet, SOL account, USD account).
fn borrower(sim: &mut Sim, b: &Bank, sol: u64) -> (Pubkey, Pubkey, Pubkey) {
    let user = sim.funded_wallet(SOL);
    let sol_acct = token::create_ata(sim, &b.issuer, &user, &b.sol_mint);
    let usd_acct = token::create_ata(sim, &b.issuer, &user, &b.usd_mint);
    token::mint_to(sim, &b.sol_mint, &b.issuer, &sol_acct, sol);
    sim.process_ix(lending::deposit(&user, &b.keys, &sol_acct, sol), &[user])
        .unwrap();
    (user, sol_acct, usd_acct)
}

/// Republish both prices now (they go stale after 60 s).
fn prices(sim: &mut Sim, b: &Bank, sol_price: u64) {
    sim.process_ix(
        oracle::publish(&b.sol_feed_authority, 1, sol_price as i64, 10_000),
        &[b.sol_feed_authority],
    )
    .unwrap();
    sim.process_ix(
        oracle::publish(&b.usd_feed_authority, 2, USD as i64, 100),
        &[b.usd_feed_authority],
    )
    .unwrap();
}

fn obligation(sim: &Sim, b: &Bank, owner: &Pubkey) -> Obligation {
    Obligation::deserialize(&mut sim.data(&lending::obligation_address(&b.keys.market, owner)))
        .unwrap()
}

fn market(sim: &Sim, b: &Bank) -> Market {
    Market::deserialize(&mut sim.data(&b.keys.market)).unwrap()
}

#[test]
fn ex5_init_market_validates() {
    let mut sim = sim();
    let b = bank(&mut sim);
    let m = market(&sim, &b);
    assert_eq!(
        (
            m.collateral_decimals,
            m.borrow_decimals,
            m.borrow_index,
            m.total_supplied
        ),
        (9, 6, WAD, 1_000_000 * USD)
    );
    let (x, y) = (
        token::create_mint(&mut sim, &b.issuer, &b.issuer, 0),
        token::create_mint(&mut sim, &b.issuer, &b.issuer, 0),
    );
    let k = MarketKeys::new(&x, &y, &b.keys.collateral_feed, &b.keys.borrow_feed);
    let bad = MarketParams {
        liquidation_threshold_bps: 7_000,
        ..PARAMS
    }; // below the LTV
    assert_error(
        sim.process_ix(
            lending::init_market(&b.issuer, &x, &y, &k, bad),
            &[b.issuer],
        ),
        LendingError::BadParams,
    );
    let fake_feed = Pubkey::new_unique();
    sim.set_account(
        fake_feed,
        Account::new_rent_exempt(vec![0; PriceFeed::SPACE], Pubkey::new_unique()),
    );
    let k = MarketKeys::new(&x, &y, &fake_feed, &b.keys.borrow_feed);
    assert_error(
        sim.process_ix(
            lending::init_market(&b.issuer, &x, &y, &k, PARAMS),
            &[b.issuer],
        ),
        OracleError::NotAFeed,
    );
}

#[test]
fn ex5_borrow_up_to_the_ltv() {
    let mut sim = sim();
    let b = bank(&mut sim);
    let (user, _, usd) = borrower(&mut sim, &b, 10 * ONE_SOL); // $1000
    assert_eq!(obligation(&sim, &b, &user).collateral, 10 * ONE_SOL);
    sim.process_ix(lending::borrow(&user, &b.keys, &usd, 700 * USD), &[user])
        .unwrap();
    assert_eq!(token::balance(&sim, &usd), 700 * USD);
    assert_error(
        sim.process_ix(lending::borrow(&user, &b.keys, &usd, 51 * USD), &[user]),
        LendingError::ExceedsLtv,
    );
    sim.process_ix(lending::borrow(&user, &b.keys, &usd, 50 * USD), &[user])
        .unwrap(); // exactly 75%
    let (thief, _, thief_usd) = borrower(&mut sim, &b, 1);
    let mut on_someone_else = lending::borrow(&thief, &b.keys, &thief_usd, USD);
    on_someone_else.accounts[2].pubkey = lending::obligation_address(&b.keys.market, &user);
    assert_error(
        sim.process_ix(on_someone_else, &[thief]),
        LendingError::WrongAccount,
    );
}

#[test]
fn ex5_interest_accrues_and_repaying_clears_the_debt() {
    let mut sim = sim();
    let b = bank(&mut sim);
    let (user, _, usd) = borrower(&mut sim, &b, 10 * ONE_SOL);
    sim.process_ix(lending::borrow(&user, &b.keys, &usd, 500 * USD), &[user])
        .unwrap();
    sim.advance_time(lmath::SECONDS_PER_YEAR as i64);
    prices(&mut sim, &b, 100 * USD);
    // touching the market accrues: borrow 1 more to see it
    sim.process_ix(lending::borrow(&user, &b.keys, &usd, USD), &[user])
        .unwrap();
    let m = market(&sim, &b);
    // utilization 0.05% -> about 2% a year
    assert!(
        m.borrow_index > WAD + WAD / 100 * 2 - WAD / 1000 && m.borrow_index < WAD + WAD / 100 * 3,
        "{}",
        m.borrow_index
    );
    let debt =
        lmath::current_debt(obligation(&sim, &b, &user).debt_scaled, m.borrow_index).unwrap();
    assert!(debt > 511 * USD && debt < 517 * USD, "{debt}");

    // repaying more than owed repays exactly the debt
    token::mint_to(&mut sim, &b.usd_mint, &b.issuer, &usd, 100 * USD);
    let before = token::balance(&sim, &usd);
    sim.process_ix(
        lending::repay(&user, &user, &b.keys, &usd, 10_000 * USD),
        &[user],
    )
    .unwrap();
    assert_eq!(obligation(&sim, &b, &user).debt_scaled, 0);
    assert_eq!(before - token::balance(&sim, &usd), debt);
}

#[test]
fn ex5_withdraw_only_what_keeps_the_position_safe() {
    let mut sim = sim();
    let b = bank(&mut sim);
    let (user, sol, usd) = borrower(&mut sim, &b, 10 * ONE_SOL);
    sim.process_ix(lending::borrow(&user, &b.keys, &usd, 600 * USD), &[user])
        .unwrap();
    // $600 needs $800 of collateral at 75%: 2 SOL may go, not 3
    assert_error(
        sim.process_ix(
            lending::withdraw(&user, &b.keys, &sol, 3 * ONE_SOL),
            &[user],
        ),
        LendingError::WouldBeUnhealthy,
    );
    sim.process_ix(
        lending::withdraw(&user, &b.keys, &sol, 2 * ONE_SOL),
        &[user],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &sol), 2 * ONE_SOL);
    sim.process_ix(
        lending::repay(&user, &user, &b.keys, &usd, 600 * USD),
        &[user],
    )
    .unwrap();
    sim.process_ix(
        lending::withdraw(&user, &b.keys, &sol, 8 * ONE_SOL),
        &[user],
    )
    .unwrap();
    assert_error(
        sim.process_ix(lending::withdraw(&user, &b.keys, &sol, 1), &[user]),
        LendingError::InsufficientCollateral,
    );
}

#[test]
fn ex5_liquidation_after_a_price_drop() {
    let mut sim = sim();
    let b = bank(&mut sim);
    let (user, _, usd) = borrower(&mut sim, &b, 10 * ONE_SOL);
    sim.process_ix(lending::borrow(&user, &b.keys, &usd, 700 * USD), &[user])
        .unwrap();
    let liquidator = sim.funded_wallet(SOL);
    let l_usd = token::create_ata(&mut sim, &b.issuer, &liquidator, &b.usd_mint);
    let l_sol = token::create_ata(&mut sim, &b.issuer, &liquidator, &b.sol_mint);
    token::mint_to(&mut sim, &b.usd_mint, &b.issuer, &l_usd, 1_000 * USD);

    // healthy at $100: 1000 * 0.8 / 700 > 1
    let liquidate = lending::liquidate(&liquidator, &user, &b.keys, &l_usd, &l_sol, u64::MAX);
    assert_error(
        sim.process_ix(liquidate.clone(), &[liquidator]),
        LendingError::Healthy,
    );

    prices(&mut sim, &b, 80 * USD); // 800 * 0.8 / 700 < 1
    sim.process_ix(liquidate, &[liquidator]).unwrap();
    assert_eq!(
        token::balance(&sim, &l_usd),
        650 * USD,
        "repaid half the debt"
    );
    assert_eq!(
        token::balance(&sim, &l_sol),
        4_593_750_000,
        "$350 + 5% of SOL at $80"
    );
    let position = obligation(&sim, &b, &user);
    assert_eq!(position.collateral, 10 * ONE_SOL - 4_593_750_000);
    assert_eq!(
        lmath::current_debt(position.debt_scaled, market(&sim, &b).borrow_index),
        Some(350 * USD)
    );
}

#[test]
fn ex5_stale_or_wrong_prices_are_refused() {
    let mut sim = sim();
    let b = bank(&mut sim);
    let (user, _, usd) = borrower(&mut sim, &b, 10 * ONE_SOL);
    sim.advance_time(61);
    assert_error(
        sim.process_ix(lending::borrow(&user, &b.keys, &usd, USD), &[user]),
        OracleError::StalePrice,
    );
    prices(&mut sim, &b, 100 * USD);
    // a feed the attacker controls, reporting SOL at $1M
    let (_, rigged) = feed(&mut sim, 7, 1_000_000 * USD as i64, 0);
    let mut ix = lending::borrow(&user, &b.keys, &usd, 100_000 * USD);
    ix.accounts[5].pubkey = rigged;
    assert_error(sim.process_ix(ix, &[user]), LendingError::WrongAccount);
}

#[test]
fn ex5_cannot_borrow_more_than_the_market_holds() {
    let mut sim = sim();
    let b = bank(&mut sim);
    let (whale, _, usd) = borrower(&mut sim, &b, 100_000 * ONE_SOL); // $10M of collateral
    assert_error(
        sim.process_ix(
            lending::borrow(&whale, &b.keys, &usd, 1_000_001 * USD),
            &[whale],
        ),
        LendingError::InsufficientLiquidity,
    );
    sim.process_ix(
        lending::borrow(&whale, &b.keys, &usd, 1_000_000 * USD),
        &[whale],
    )
    .unwrap();
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_loose_slippage_gets_sandwiched() {
    let (ra, rb, fee) = (1_000_000_000, 1_000_000_000, 30);
    let victim_in = 50_000_000;
    let loose = sandwich::min_out_with_slippage(victim_in, ra, rb, fee, 1_000); // 10%
    let sizes: Vec<u64> = (1..=40).map(|i| i * 10_000_000).collect();
    let (size, attack) =
        sandwich::best_attack(ra, rb, fee, victim_in, loose, &sizes).expect("profitable");
    assert!(attack.profit > 0 && size > 0);
    let honest = amm_math::swap_out(victim_in, ra, rb, fee).unwrap();
    assert!(attack.victim_out < honest, "the victim got a worse price");
    assert!(attack.victim_out >= loose);
}

#[test]
fn bonus_tight_slippage_stops_it() {
    let (ra, rb, fee) = (1_000_000_000, 1_000_000_000, 30);
    let victim_in = 50_000_000;
    let tight = sandwich::min_out_with_slippage(victim_in, ra, rb, fee, 10); // 0.1%
    let sizes: Vec<u64> = (1..=40).map(|i| i * 10_000_000).collect();
    assert_eq!(
        sandwich::best_attack(ra, rb, fee, victim_in, tight, &sizes),
        None
    );
    // a big front-run simply makes the victim's swap fail
    assert_eq!(
        sandwich::sandwich(ra, rb, fee, victim_in, tight, 100_000_000),
        None
    );
    assert_eq!(
        sandwich::min_out_with_slippage(1_000, 1_000_000, 1_000_000, 0, 0),
        amm_math::swap_out(1_000, 1_000_000, 1_000_000, 0).unwrap()
    );
}
