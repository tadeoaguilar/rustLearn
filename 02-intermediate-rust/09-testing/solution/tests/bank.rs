//! Integration test: uses only the public API, like any other crate would.

mod common;

use m09_testing_solution::account::{AccountError, transfer};

/// Splits a Vec so we can hold two `&mut` elements at once -- `&mut v[0]` and
/// `&mut v[1]` together would be two mutable borrows of `v`.
fn pair(
    v: &mut [m09_testing_solution::account::Account],
    i: usize,
    j: usize,
) -> (
    &mut m09_testing_solution::account::Account,
    &mut m09_testing_solution::account::Account,
) {
    assert!(i < j);
    let (left, right) = v.split_at_mut(j);
    (&mut left[i], &mut right[0])
}

#[test]
fn money_is_conserved_across_many_transfers() {
    let mut accounts = common::three_accounts();
    let before = common::total(&accounts);

    let plan = [(0, 1, 300), (1, 2, 700), (0, 2, 100), (0, 1, 5_000)]; // the last one fails
    for (from, to, amount) in plan {
        let (a, b) = pair(&mut accounts, from, to);
        let _ = transfer(a, b, amount);
    }

    assert_eq!(common::total(&accounts), before);
    let balances: Vec<u64> = accounts.iter().map(|a| a.balance()).collect();
    assert_eq!(balances, vec![600, 100, 800]);
}

#[test]
fn money_is_conserved_when_the_target_is_frozen() {
    let mut accounts = vec![common::three_accounts().remove(0), common::frozen("Dan", 0)];
    let before = common::total(&accounts);
    let (a, b) = pair(&mut accounts, 0, 1);
    assert_eq!(transfer(a, b, 100), Err(AccountError::Frozen));
    assert_eq!(common::total(&accounts), before);
}
