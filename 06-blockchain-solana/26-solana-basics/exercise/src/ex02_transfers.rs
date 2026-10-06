//! Exercise 2: funding a wallet, transfers and rent.
//!
//! `Sim` stands in for an RPC connection to a cluster: `airdrop` is the
//! devnet faucet, `process` is `send_and_confirm_transaction`.

use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_sdk_ids::{bpf_loader_upgradeable, system_program, sysvar};
use solana_system_interface::instruction as system_instruction;
use solsim::{Account, Sim, TxError};

/// What kind of account this is, judging only by its fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountKind {
    /// System-owned, no data: holds lamports, can pay and sign.
    Wallet,
    /// Executable: a deployed program.
    Program,
    /// Owned by the Sysvar program: cluster state like the clock.
    Sysvar,
    /// Holds data for the program that owns it.
    Data { owner: Pubkey },
}

pub fn describe(account: &Account) -> AccountKind {
    todo!("Exercise 2")
}

/// A System program transfer instruction.
pub fn transfer_ix(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    todo!("Exercise 2")
}

/// Pay several people in ONE transaction: either everyone is paid or nobody.
pub fn pay_many(sim: &mut Sim, payer: &Pubkey, payments: &[(Pubkey, u64)]) -> Result<(), TxError> {
    todo!("Exercise 2")
}

/// How much `wallet` can send while staying alive (rent-exempt). Sending
/// everything is also allowed -- the account is then deleted.
pub fn spendable(sim: &Sim, wallet: &Pubkey) -> u64 {
    todo!("Exercise 2")
}

/// Create an account at `new_account` with `space` bytes of data, owned by
/// `owner`, funded with exactly the rent-exempt minimum. Both `payer` and
/// `new_account` must sign: nobody can create an account at an address they
/// don't hold the key for (PDAs are the exception, Exercise 5).
pub fn create_data_account(
    sim: &mut Sim,
    payer: &Pubkey,
    new_account: &Pubkey,
    space: usize,
    owner: &Pubkey,
) -> Result<(), TxError> {
    todo!("Exercise 2")
}
