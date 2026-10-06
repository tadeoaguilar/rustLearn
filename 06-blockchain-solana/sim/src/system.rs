//! The System program: the only program that can create accounts, and the
//! owner of every wallet. This covers the instructions Phase 6 uses.

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_sdk_ids::system_program;

/// `SystemError` codes, as the real program returns them.
const ACCOUNT_ALREADY_IN_USE: u32 = 0;
const RESULT_WITH_NEGATIVE_LAMPORTS: u32 = 1;
const INVALID_ACCOUNT_DATA_LENGTH: u32 = 3;

/// The real limit is 10 MiB; solsim accounts grow at most 10 KiB per
/// instruction (see `runtime::Slot`).
const MAX_SPACE: u64 = 10 * 1024;

pub(crate) fn process<'info>(
    _program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    let mut r = Reader(data);
    match r.u32()? {
        // CreateAccount { lamports, space, owner }: [funder, new account]
        0 => {
            let (lamports, space, owner) = (r.u64()?, r.u64()?, r.pubkey()?);
            let [from, to, ..] = accounts else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            if !to.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            if to.lamports() > 0 || !to.data_is_empty() || *to.owner != system_program::ID {
                return Err(ProgramError::Custom(ACCOUNT_ALREADY_IN_USE));
            }
            allocate(to, space)?;
            to.assign(&owner);
            transfer(from, to, lamports)
        }
        // Assign { owner }: [account]
        1 => {
            let owner = r.pubkey()?;
            let account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
            if !account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            account.assign(&owner);
            Ok(())
        }
        // Transfer { lamports }: [from, to]
        2 => {
            let lamports = r.u64()?;
            let [from, to, ..] = accounts else {
                return Err(ProgramError::NotEnoughAccountKeys);
            };
            transfer(from, to, lamports)
        }
        // Allocate { space }: [account]
        8 => {
            let space = r.u64()?;
            let account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;
            if !account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            allocate(account, space)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn allocate(account: &AccountInfo, space: u64) -> ProgramResult {
    if !account.data_is_empty() || *account.owner != system_program::ID {
        return Err(ProgramError::Custom(ACCOUNT_ALREADY_IN_USE));
    }
    if space > MAX_SPACE {
        return Err(ProgramError::Custom(INVALID_ACCOUNT_DATA_LENGTH));
    }
    account.resize(space as usize)
}

fn transfer(from: &AccountInfo, to: &AccountInfo, lamports: u64) -> ProgramResult {
    if !from.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // Only plain wallets can pay: a System-owned account without data.
    if !from.data_is_empty() || *from.owner != system_program::ID {
        return Err(ProgramError::InvalidArgument);
    }
    let remaining = from
        .lamports()
        .checked_sub(lamports)
        .ok_or(ProgramError::Custom(RESULT_WITH_NEGATIVE_LAMPORTS))?;
    **from.try_borrow_mut_lamports()? = remaining;
    let credited = to
        .lamports()
        .checked_add(lamports)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **to.try_borrow_mut_lamports()? = credited;
    Ok(())
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], ProgramError> {
        let (head, rest) = self
            .0
            .split_first_chunk::<N>()
            .ok_or(ProgramError::InvalidInstructionData)?;
        self.0 = rest;
        Ok(*head)
    }
    fn u32(&mut self) -> Result<u32, ProgramError> {
        self.take().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> Result<u64, ProgramError> {
        self.take().map(u64::from_le_bytes)
    }
    fn pubkey(&mut self) -> Result<Pubkey, ProgramError> {
        self.take().map(Pubkey::new_from_array)
    }
}
