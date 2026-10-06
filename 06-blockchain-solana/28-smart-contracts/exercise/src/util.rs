//! Helpers every program in this module uses (provided -- module 26 taught
//! each of them): creating and closing PDA accounts, token CPIs, and
//! loading token accounts safely.

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

/// Create `account` (a PDA of `owner` with `seeds`, bump included) with
/// `space` bytes, funded to rent exemption by `payer`.
pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    space: usize,
    owner: &Pubkey,
    seeds: &[&[u8]],
) -> ProgramResult {
    if *system_program.key != solana_system_interface::program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    let lamports = Rent::get()?.minimum_balance(space);
    let ix = solana_system_interface::instruction::create_account(
        payer.key,
        account.key,
        lamports,
        space as u64,
        owner,
    );
    invoke_signed(
        &ix,
        &[payer.clone(), account.clone(), system_program.clone()],
        &[seeds],
    )
}

/// Close an account this program owns: all lamports to `destination`, data
/// wiped, ownership back to the System program.
pub fn close_account(account: &AccountInfo, destination: &AccountInfo) -> ProgramResult {
    let lamports = account.lamports();
    **account.try_borrow_mut_lamports()? = 0;
    **destination.try_borrow_mut_lamports()? = destination
        .lamports()
        .checked_add(lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    account.resize(0)?;
    account.assign(&solana_system_interface::program::ID);
    Ok(())
}

/// Check that `token_program` is the SPL Token program (never CPI into a
/// program id a caller handed you without checking it).
pub fn check_token_program(token_program: &AccountInfo) -> ProgramResult {
    if *token_program.key != spl_token_interface::ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

/// Decode a token account, checking it really is one (owned by the token program).
pub fn token_account(
    info: &AccountInfo,
) -> Result<spl_token_interface::state::Account, ProgramError> {
    if *info.owner != spl_token_interface::ID {
        return Err(ProgramError::IllegalOwner);
    }
    spl_token_interface::state::Account::unpack(&info.try_borrow_data()?)
}

/// Decode a mint, checking it really is one.
pub fn mint(info: &AccountInfo) -> Result<spl_token_interface::state::Mint, ProgramError> {
    if *info.owner != spl_token_interface::ID {
        return Err(ProgramError::IllegalOwner);
    }
    spl_token_interface::state::Mint::unpack(&info.try_borrow_data()?)
}

/// SPL `transfer` of `amount` from `source` to `destination`, authorised by
/// `authority` -- a signer, or a PDA of the caller when `seeds` is non-empty.
pub fn token_transfer<'a>(
    token_program: &AccountInfo<'a>,
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    check_token_program(token_program)?;
    let ix = spl_token_interface::instruction::transfer(
        &spl_token_interface::ID,
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;
    invoke_signed(
        &ix,
        &[
            source.clone(),
            destination.clone(),
            authority.clone(),
            token_program.clone(),
        ],
        seeds,
    )
}

/// SPL `mint_to`, authorised by `authority` (signer or PDA via `seeds`).
pub fn token_mint_to<'a>(
    token_program: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    check_token_program(token_program)?;
    let ix = spl_token_interface::instruction::mint_to(
        &spl_token_interface::ID,
        mint.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;
    invoke_signed(
        &ix,
        &[
            mint.clone(),
            destination.clone(),
            authority.clone(),
            token_program.clone(),
        ],
        seeds,
    )
}

/// Create a token account for `mint` at the PDA `account` (with `seeds`),
/// owned (token-wise) by `authority`.
pub fn create_token_account<'a>(
    payer: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &Pubkey,
    system_program: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    seeds: &[&[u8]],
) -> ProgramResult {
    check_token_program(token_program)?;
    create_pda_account(
        payer,
        account,
        system_program,
        spl_token_interface::state::Account::LEN,
        &spl_token_interface::ID,
        seeds,
    )?;
    let ix = spl_token_interface::instruction::initialize_account3(
        &spl_token_interface::ID,
        account.key,
        mint.key,
        authority,
    )?;
    invoke_signed(
        &ix,
        &[account.clone(), mint.clone(), token_program.clone()],
        &[],
    )
}

/// Close a token account (it must be empty), rent to `destination`,
/// authorised by `authority` (signer or PDA via `seeds`).
pub fn close_token_account<'a>(
    token_program: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    check_token_program(token_program)?;
    let ix = spl_token_interface::instruction::close_account(
        &spl_token_interface::ID,
        account.key,
        destination.key,
        authority.key,
        &[],
    )?;
    invoke_signed(
        &ix,
        &[
            account.clone(),
            destination.clone(),
            authority.clone(),
            token_program.clone(),
        ],
        seeds,
    )
}

/// Fail with `MissingRequiredSignature` unless `info` signed.
pub fn require_signer(info: &AccountInfo) -> ProgramResult {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

/// Fail with `InvalidSeeds` unless `info` is the PDA for `seeds` (no bump)
/// and return its bump.
pub fn require_pda(
    info: &AccountInfo,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<u8, ProgramError> {
    let (expected, bump) = Pubkey::find_program_address(seeds, program_id);
    if *info.key != expected {
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump)
}
