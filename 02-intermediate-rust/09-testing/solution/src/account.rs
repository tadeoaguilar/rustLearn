//! Bank accounts. Amounts are in cents.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountError {
    InsufficientFunds { balance: u64, requested: u64 },
    Frozen,
    ZeroAmount,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountError::InsufficientFunds { balance, requested } => {
                write!(
                    f,
                    "insufficient funds: balance {balance}, requested {requested}"
                )
            }
            AccountError::Frozen => write!(f, "account is frozen"),
            AccountError::ZeroAmount => write!(f, "amount must be positive"),
        }
    }
}

impl std::error::Error for AccountError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    owner: String,
    balance: u64,
    frozen: bool,
}

impl Account {
    /// # Panics
    /// If `owner` is empty -- that's a programming error, not user input.
    pub fn new(owner: &str, initial_balance: u64) -> Self {
        assert!(!owner.trim().is_empty(), "owner name must not be empty");
        Account {
            owner: owner.to_string(),
            balance: initial_balance,
            frozen: false,
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
        self.frozen = false;
    }

    /// Checks that this account could accept a deposit, without making one.
    fn check_deposit(&self, amount: u64) -> Result<(), AccountError> {
        if amount == 0 {
            return Err(AccountError::ZeroAmount);
        }
        if self.frozen {
            return Err(AccountError::Frozen);
        }
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<(), AccountError> {
        self.check_deposit(amount)?;
        self.balance += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<(), AccountError> {
        if amount == 0 {
            return Err(AccountError::ZeroAmount);
        }
        if self.frozen {
            return Err(AccountError::Frozen);
        }
        if amount > self.balance {
            return Err(AccountError::InsufficientFunds {
                balance: self.balance,
                requested: amount,
            });
        }
        self.balance -= amount;
        Ok(())
    }
}

/// Moves `amount` from one account to the other. Either both balances change
/// or neither does.
///
/// BUG FIXED: the original withdrew from `from`, then tried to deposit into
/// `to`. If `to` was frozen the deposit failed -- after the money had already
/// left `from`. Now the target is checked *before* anything changes.
pub fn transfer(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), AccountError> {
    to.check_deposit(amount)?;
    from.withdraw(amount)?;
    to.deposit(amount).expect("checked above");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "owner name must not be empty")]
    fn new_rejects_empty_owner() {
        Account::new("  ", 0);
    }

    #[test]
    fn deposit_then_withdraw() -> Result<(), AccountError> {
        // A test can return Result and use `?`; an Err fails the test.
        let mut acc = Account::new("Ann", 0);
        acc.deposit(50)?;
        acc.withdraw(20)?;
        assert_eq!(acc.balance(), 30);
        Ok(())
    }

    #[test]
    fn withdraw_more_than_balance_is_an_error() {
        let mut acc = Account::new("Ann", 100);
        assert_eq!(
            acc.withdraw(150),
            Err(AccountError::InsufficientFunds {
                balance: 100,
                requested: 150
            })
        );
        assert_eq!(
            acc.balance(),
            100,
            "a failed withdrawal must not change the balance"
        );
    }

    #[test]
    fn withdrawing_everything_is_fine() {
        let mut acc = Account::new("Ann", 100);
        assert_eq!(acc.withdraw(100), Ok(()));
        assert_eq!(acc.balance(), 0);
    }

    #[test]
    fn zero_amounts_are_rejected() {
        let mut acc = Account::new("Ann", 100);
        assert_eq!(acc.deposit(0), Err(AccountError::ZeroAmount));
        assert_eq!(acc.withdraw(0), Err(AccountError::ZeroAmount));
    }

    #[test]
    fn frozen_accounts_reject_deposits_and_withdrawals() {
        let mut acc = Account::new("Ann", 100);
        acc.freeze();
        assert!(matches!(acc.deposit(1), Err(AccountError::Frozen)));
        assert!(matches!(acc.withdraw(1), Err(AccountError::Frozen)));
        assert_eq!(acc.balance(), 100, "but the balance is still readable");
        acc.unfreeze();
        assert!(acc.withdraw(1).is_ok());
    }

    #[test]
    fn transfer_moves_money() {
        let (mut a, mut b) = (Account::new("A", 100), Account::new("B", 0));
        transfer(&mut a, &mut b, 30).unwrap();
        assert_eq!((a.balance(), b.balance()), (70, 30));
    }

    #[test]
    fn failed_transfer_to_frozen_account_changes_nothing() {
        // Bug #3: this is the test that finds it.
        let (mut a, mut b) = (Account::new("A", 100), Account::new("B", 0));
        b.freeze();
        assert_eq!(transfer(&mut a, &mut b, 30), Err(AccountError::Frozen));
        assert_eq!(a.balance(), 100, "money must not leave the source");
        assert_eq!(b.balance(), 0);
    }

    #[test]
    fn failed_transfer_for_lack_of_funds_changes_nothing() {
        let (mut a, mut b) = (Account::new("A", 10), Account::new("B", 0));
        assert!(matches!(
            transfer(&mut a, &mut b, 30),
            Err(AccountError::InsufficientFunds { .. })
        ));
        assert_eq!((a.balance(), b.balance()), (10, 0));
    }

    #[test]
    fn error_messages_are_readable() {
        let e = AccountError::InsufficientFunds {
            balance: 1,
            requested: 2,
        };
        assert_eq!(e.to_string(), "insufficient funds: balance 1, requested 2");
    }
}
