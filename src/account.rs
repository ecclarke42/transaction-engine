use serde::Serialize;

use crate::{Amount, ClientId};

#[derive(Debug, Default)]
pub struct Account {
    available: Amount,
    held: Amount,

    locked: bool,
}

impl Account {
    /// Get the amount of available funds in the account
    pub fn available_funds(&self) -> Amount {
        self.available
    }

    /// Get the amount of funds in the account placed under hold
    pub fn held_funds(&self) -> Amount {
        self.held
    }

    /// Get the total funds in the account (available and held)
    pub fn total_funds(&self) -> Amount {
        self.available + self.held
    }

    /// Check if the account is locked or frozen
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Deposit an amount into the account, if it isn't locked
    ///
    /// Deposit amounts must be positive
    pub fn deposit(&mut self, amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::Locked);
        }

        if amount.is_sign_negative() {
            return Err(AccountError::NegativeAmount);
        }
        self.available += amount;
        Ok(())
    }

    /// Withdraw an amount from the account, if the funds are available and the
    /// account isn't locked.
    ///
    /// Withdrawal amounts must be positive
    pub fn withdraw(&mut self, amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if amount.is_sign_negative() {
            return Err(AccountError::NegativeAmount);
        }
        if amount > self.available {
            return Err(AccountError::InsufficientFunds);
        }
        self.available -= amount;
        Ok(())
    }

    /// Add a hold on some funds from the account, if the funds are available
    /// and the account isn't locked.
    ///
    /// Held amounts must be positive
    pub fn hold(&mut self, amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if amount.is_sign_negative() {
            return Err(AccountError::NegativeAmount);
        }
        if amount > self.available {
            return Err(AccountError::InsufficientFunds);
        }
        self.available -= amount;
        self.held += amount;
        Ok(())
    }

    /// Release held funds in the account, if the funds are available and the
    /// account isn't locked.
    ///
    /// Release amounts must be positive
    pub fn release(&mut self, amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if amount.is_sign_negative() {
            return Err(AccountError::NegativeAmount);
        }
        if amount > self.held {
            return Err(AccountError::InsufficientFunds);
        }
        self.held -= amount;
        self.available += amount;
        Ok(())
    }

    /// Clear held funds from the account, but do not return them to the
    /// account's available funds.
    pub fn chargeback(&mut self, amount: Amount) -> Result<(), AccountError> {
        if self.locked {
            return Err(AccountError::Locked);
        }
        if amount.is_sign_negative() {
            return Err(AccountError::NegativeAmount);
        }
        if amount > self.held {
            return Err(AccountError::InsufficientFunds);
        }
        self.held -= amount;
        Ok(())
    }

    /// Lock an account
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Unlock an account
    pub fn unlock(&mut self) {
        self.locked = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AccountError {
    #[error("the account is locked")]
    Locked,

    #[error("there are not enough funds to withdraw")]
    InsufficientFunds,

    #[error("cannot deposit or withdraw a negative amount")]
    NegativeAmount,
}

/// Serializable account data
#[derive(Debug, Serialize)]
pub struct AccountData {
    pub client: ClientId,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

#[cfg(feature = "decimal")]
impl From<(&ClientId, &Account)> for AccountData {
    fn from((id, account): (&ClientId, &Account)) -> Self {
        use rust_decimal::prelude::*;
        let strategy = RoundingStrategy::MidpointAwayFromZero;
        Self {
            client: *id,
            available: account
                .available_funds()
                .round_dp_with_strategy(4, strategy)
                .normalize(),

            held: account
                .held_funds()
                .round_dp_with_strategy(4, strategy)
                .normalize(),

            total: account
                .total_funds()
                .round_dp_with_strategy(4, strategy)
                .normalize(),

            locked: account.is_locked(),
        }
    }
}

#[cfg(not(feature = "decimal"))]
impl From<(&ClientId, &Account)> for AccountData {
    fn from((id, account): (&ClientId, &Account)) -> Self {
        Self {
            client: *id,
            available: account.available_funds(),
            held: account.held_funds(),
            total: account.total_funds(),
            locked: account.is_locked(),
        }
    }
}
