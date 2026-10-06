//! The optional on-chain track: run programs built with `cargo build-sbf`
//! in LiteSVM, the real Solana VM and runtime in a library.
//!
//! Same instructions, same checks as the native tests -- but now the
//! program is SBF bytecode, transactions are really signed, and compute
//! units are metered. Build the programs first with `./build-sbf.sh`.

use std::path::PathBuf;

use litesvm::LiteSVM;
use solana_instruction_error::InstructionError;
use solana_keypair::Keypair;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;

use crate::Account;

/// Where `./build-sbf.sh <which>` put `<name>.so` (`which` is "solution" or "mine").
pub fn program_path(which: &str, name: &str) -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../target")));
    target.join("deploy").join(which).join(format!("{name}.so"))
}

/// A failed transaction on the VM.
#[derive(Debug, Clone)]
pub struct VmError {
    pub error: TransactionError,
    pub logs: Vec<String>,
}

impl VmError {
    /// The program's custom error code, if that's why it failed.
    pub fn custom_code(&self) -> Option<u32> {
        match self.error {
            TransactionError::InstructionError(_, InstructionError::Custom(code)) => Some(code),
            _ => None,
        }
    }
}

/// What a successful transaction produced.
#[derive(Debug, Clone)]
pub struct VmMeta {
    pub logs: Vec<String>,
    pub return_data: Option<(Pubkey, Vec<u8>)>,
    pub compute_units: u64,
}

pub struct Vm {
    pub svm: LiteSVM,
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm {
    /// A fresh VM with the System, SPL Token and ATA programs.
    pub fn new() -> Self {
        Vm {
            svm: LiteSVM::new(),
        }
    }

    /// Deploy `target/deploy/<which>/<name>.so` at `program_id`.
    pub fn load(&mut self, program_id: Pubkey, which: &str, name: &str) {
        let path = program_path(which, name);
        assert!(
            path.exists(),
            "{} is missing: build the programs first with `./build-sbf.sh {which}` (in 06-blockchain-solana/)",
            path.display()
        );
        self.svm
            .add_program_from_file(program_id, &path)
            .expect("load program");
    }

    /// A new keypair holding `lamports`.
    pub fn wallet(&mut self, lamports: u64) -> Keypair {
        let keypair = Keypair::new();
        self.svm
            .airdrop(&keypair.pubkey(), lamports)
            .expect("airdrop");
        keypair
    }

    /// Sign and send a transaction; the first signer pays the fee.
    pub fn process(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<VmMeta, VmError> {
        let payer = signers.first().expect("a fee payer").pubkey();
        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(&payer),
            signers,
            self.svm.latest_blockhash(),
        );
        let result = self.svm.send_transaction(tx);
        // A new blockhash, so sending the same instructions again is a new
        // transaction rather than a duplicate.
        self.svm.expire_blockhash();
        match result {
            Ok(meta) => Ok(VmMeta {
                logs: meta.logs,
                return_data: (!meta.return_data.data.is_empty())
                    .then_some((meta.return_data.program_id, meta.return_data.data)),
                compute_units: meta.compute_units_consumed,
            }),
            Err(failed) => Err(VmError {
                error: failed.err,
                logs: failed.meta.logs,
            }),
        }
    }

    pub fn account(&self, key: &Pubkey) -> Option<Account> {
        self.svm
            .get_account(key)
            .filter(|a| a.lamports > 0)
            .map(|a| Account {
                lamports: a.lamports,
                data: a.data,
                owner: a.owner,
                executable: a.executable,
            })
    }

    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.account(key).map_or(0, |a| a.lamports)
    }

    pub fn data(&self, key: &Pubkey) -> Vec<u8> {
        self.account(key).map_or_else(Vec::new, |a| a.data)
    }

    pub fn minimum_balance(&self, data_len: usize) -> u64 {
        self.svm.minimum_balance_for_rent_exemption(data_len)
    }
}
