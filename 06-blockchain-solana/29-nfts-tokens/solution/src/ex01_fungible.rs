//! Exercise 1: a fungible token, from the client side.
//!
//! The SPL Token program is shared by every token on Solana: a *mint*
//! account defines a token (supply, decimals, who may mint and freeze), and
//! *token accounts* hold balances of one mint for one owner. An owner's
//! canonical token account for a mint is its *associated token account*
//! (ATA), a PDA anyone can compute. No code to deploy: a token is two
//! instructions to an existing program.

use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_system_interface::instruction::create_account;
use solsim::ata::{create_associated_token_account_idempotent, get_associated_token_address};
use solsim::{Sim, TxError};
use spl_token_interface::instruction as token_ix;
use spl_token_interface::instruction::AuthorityType;
use spl_token_interface::state::{Account as TokenAccount, Mint};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not an amount with at most {decimals} decimals: {text:?}")]
pub struct AmountError {
    pub text: String,
    pub decimals: u8,
}

/// The two instructions that make a mint: allocate it (owned by the token
/// program, rent-exempt), then initialize it.
pub fn create_mint_ixs(
    payer: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    lamports: u64,
) -> [Instruction; 2] {
    [
        create_account(
            payer,
            mint,
            lamports,
            Mint::LEN as u64,
            &spl_token_interface::ID,
        ),
        token_ix::initialize_mint2(
            &spl_token_interface::ID,
            mint,
            mint_authority,
            freeze_authority,
            decimals,
        )
        .expect("valid mint instruction"),
    ]
}

/// Create a new token: `payer` pays and becomes mint authority (and freeze
/// authority if `freezable`). Returns the mint's address.
pub fn create_token(
    sim: &mut Sim,
    payer: &Pubkey,
    decimals: u8,
    freezable: bool,
) -> Result<Pubkey, TxError> {
    let mint = Pubkey::new_unique();
    let freeze = freezable.then_some(payer);
    let ixs = create_mint_ixs(
        payer,
        &mint,
        payer,
        freeze,
        decimals,
        sim.minimum_balance(Mint::LEN),
    );
    // The new mint account signs its own creation (solsim takes the listed
    // signers at face value; a real client would sign with a fresh keypair).
    sim.process(&ixs, &[*payer, mint])?;
    Ok(mint)
}

/// Mint `amount` base units to `wallet`'s ATA, creating it if needed (the
/// authority pays). `mint_to_checked` also verifies the decimals. Returns the ATA.
pub fn mint_to_wallet(
    sim: &mut Sim,
    mint: &Pubkey,
    authority: &Pubkey,
    wallet: &Pubkey,
    amount: u64,
) -> Result<Pubkey, TxError> {
    let decimals = decimals(sim, mint);
    let ata = get_associated_token_address(wallet, mint);
    let ixs = [
        create_associated_token_account_idempotent(authority, wallet, mint),
        token_ix::mint_to_checked(
            &spl_token_interface::ID,
            mint,
            &ata,
            authority,
            &[],
            amount,
            decimals,
        )
        .expect("valid"),
    ];
    sim.process(&ixs, &[*authority])?;
    Ok(ata)
}

/// Send `amount` from `from`'s ATA to `to`'s ATA, creating the recipient's
/// if needed (the sender pays for it).
pub fn send(
    sim: &mut Sim,
    mint: &Pubkey,
    from: &Pubkey,
    to: &Pubkey,
    amount: u64,
) -> Result<(), TxError> {
    let decimals = decimals(sim, mint);
    let source = get_associated_token_address(from, mint);
    let destination = get_associated_token_address(to, mint);
    let ixs = [
        create_associated_token_account_idempotent(from, to, mint),
        token_ix::transfer_checked(
            &spl_token_interface::ID,
            &source,
            mint,
            &destination,
            from,
            &[],
            amount,
            decimals,
        )
        .expect("valid"),
    ];
    sim.process(&ixs, &[*from]).map(|_| ())
}

/// Destroy `amount` of `owner`'s tokens (supply goes down).
pub fn burn(sim: &mut Sim, mint: &Pubkey, owner: &Pubkey, amount: u64) -> Result<(), TxError> {
    let account = get_associated_token_address(owner, mint);
    let ix = token_ix::burn_checked(
        &spl_token_interface::ID,
        &account,
        mint,
        owner,
        &[],
        amount,
        decimals(sim, mint),
    )
    .expect("valid");
    sim.process_ix(ix, &[*owner]).map(|_| ())
}

/// Freeze `wallet`'s ATA: no transfers in or out until thawed.
pub fn freeze(
    sim: &mut Sim,
    mint: &Pubkey,
    freeze_authority: &Pubkey,
    wallet: &Pubkey,
) -> Result<(), TxError> {
    let account = get_associated_token_address(wallet, mint);
    let ix = token_ix::freeze_account(
        &spl_token_interface::ID,
        &account,
        mint,
        freeze_authority,
        &[],
    )
    .expect("valid");
    sim.process_ix(ix, &[*freeze_authority]).map(|_| ())
}

pub fn thaw(
    sim: &mut Sim,
    mint: &Pubkey,
    freeze_authority: &Pubkey,
    wallet: &Pubkey,
) -> Result<(), TxError> {
    let account = get_associated_token_address(wallet, mint);
    let ix = token_ix::thaw_account(
        &spl_token_interface::ID,
        &account,
        mint,
        freeze_authority,
        &[],
    )
    .expect("valid");
    sim.process_ix(ix, &[*freeze_authority]).map(|_| ())
}

/// Remove the mint authority for good: the supply can never grow again.
pub fn fix_supply(sim: &mut Sim, mint: &Pubkey, authority: &Pubkey) -> Result<(), TxError> {
    let ix = token_ix::set_authority(
        &spl_token_interface::ID,
        mint,
        None,
        AuthorityType::MintTokens,
        authority,
        &[],
    )
    .expect("valid");
    sim.process_ix(ix, &[*authority]).map(|_| ())
}

/// `wallet`'s balance of `mint` (0 without an ATA).
pub fn balance_of(sim: &Sim, mint: &Pubkey, wallet: &Pubkey) -> u64 {
    let ata = get_associated_token_address(wallet, mint);
    sim.account(&ata)
        .and_then(|a| TokenAccount::unpack(&a.data).ok())
        .map_or(0, |a| a.amount)
}

/// The mint's decimals (panics if `mint` isn't a mint: client-side helper).
pub fn decimals(sim: &Sim, mint: &Pubkey) -> u8 {
    Mint::unpack(sim.data(mint)).expect("a mint").decimals
}

/// Base units for people: `format_amount(1_500_000, 6) == "1.5"`.
pub fn format_amount(amount: u64, decimals: u8) -> String {
    let scale = 10u64.pow(decimals as u32);
    let (whole, frac) = (amount / scale, amount % scale);
    if frac == 0 {
        return whole.to_string();
    }
    let frac = format!("{frac:0width$}", width = decimals as usize);
    format!("{whole}.{}", frac.trim_end_matches('0'))
}

/// The inverse: `parse_amount("1.5", 6) == Ok(1_500_000)`; exact, checked.
pub fn parse_amount(text: &str, decimals: u8) -> Result<u64, AmountError> {
    let err = || AmountError {
        text: text.to_string(),
        decimals,
    };
    let (whole, frac) = text.split_once('.').unwrap_or((text, ""));
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || (text.contains('.') && !digits(frac)) || frac.len() > decimals as usize {
        return Err(err());
    }
    let scale = 10u64.pow(decimals as u32);
    let whole: u64 = whole.parse().map_err(|_| err())?;
    let frac: u64 = if frac.is_empty() {
        0
    } else {
        format!("{frac:0<width$}", width = decimals as usize)
            .parse()
            .map_err(|_| err())?
    };
    whole
        .checked_mul(scale)
        .and_then(|w| w.checked_add(frac))
        .ok_or_else(err)
}
