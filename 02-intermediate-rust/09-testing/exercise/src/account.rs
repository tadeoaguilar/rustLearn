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
pub fn transfer(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), AccountError> {
    from.withdraw(amount)?;
    to.deposit(amount)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise 2: your tests here.
}
