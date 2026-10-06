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
    if account.executable || account.owner == bpf_loader_upgradeable::ID {
        AccountKind::Program
    } else if account.owner == sysvar::ID {
        AccountKind::Sysvar
    } else if account.owner == system_program::ID && account.data.is_empty() {
        AccountKind::Wallet
    } else {
        AccountKind::Data {
            owner: account.owner,
        }
    }
}

/// A System program transfer instruction.
pub fn transfer_ix(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    system_instruction::transfer(from, to, lamports)
}

/// Pay several people in ONE transaction: either everyone is paid or nobody.
pub fn pay_many(sim: &mut Sim, payer: &Pubkey, payments: &[(Pubkey, u64)]) -> Result<(), TxError> {
    let ixs: Vec<Instruction> = payments
        .iter()
        .map(|(to, lamports)| transfer_ix(payer, to, *lamports))
        .collect();
    sim.process(&ixs, &[*payer]).map(|_| ())
}

/// How much `wallet` can send while staying alive (rent-exempt). Sending
/// everything is also allowed -- the account is then deleted.
pub fn spendable(sim: &Sim, wallet: &Pubkey) -> u64 {
    sim.lamports(wallet).saturating_sub(sim.minimum_balance(0))
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
    let lamports = sim.minimum_balance(space);
    let ix = system_instruction::create_account(payer, new_account, lamports, space as u64, owner);
    sim.process_ix(ix, &[*payer, *new_account]).map(|_| ())
}
