//! Helpers shared by the integration tests.
//!
//! In `tests/common/mod.rs`, not `tests/common.rs`: every `tests/*.rs` file is
//! compiled as its own test binary, so `common.rs` would show up in the output
//! as a test file with 0 tests. Files in subdirectories aren't.

use m09_testing_solution::account::Account;

pub fn three_accounts() -> Vec<Account> {
    vec![
        Account::new("Ann", 1_000),
        Account::new("Bob", 500),
        Account::new("Cat", 0),
    ]
}

pub fn total(accounts: &[Account]) -> u64 {
    accounts.iter().map(Account::balance).sum()
}

// Not every test file uses every helper; this keeps the others quiet.
#[allow(dead_code)]
pub fn frozen(owner: &str, balance: u64) -> Account {
    let mut a = Account::new(owner, balance);
    a.freeze();
    a
}
