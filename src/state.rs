use std::collections::{hash_map::Entry, HashMap};

use super::{Action, ActionKind, ClientId, TransactionId, TransactionState};
use crate::{account::Account, AccountData, Transaction};

/// The internal state of the engine
#[derive(Debug, Default)]
pub struct State {
    accounts: HashMap<ClientId, Account>,

    transactions: HashMap<TransactionId, Transaction>,
    /* TODO: potential improvement, track transaction ordering?
     * Esp for when a previous transaction was disputed/changed and it affects downstream
     * transaction_ordering */
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, action: Action) -> Result<(), UpdateError> {
        match action.kind {
            ActionKind::Deposit => {
                let amount = action.amount.ok_or(UpdateError::NoAmount)?;

                // TODO: I'm not super excited about the entry API/match usage for transaction
                // here (and in Withdrawal), but I think it's be two lookups to
                // do a `contains` and `insert`, so this may be better?
                let account = self.accounts.entry(action.client_id);
                let transaction = self.transactions.entry(action.transaction_id);

                // Should be a new transaction
                if matches!(transaction, Entry::Occupied(_)) {
                    return Err(UpdateError::TransactionUsed(action.transaction_id));
                }

                // Try doing the deposit
                let state = match account.or_default().deposit(amount) {
                    Ok(()) => TransactionState::Succeeded,
                    Err(e) => TransactionState::Failed(e),
                };

                // Add the transaction
                transaction.or_insert(Transaction {
                    id: action.transaction_id,
                    client: action.client_id,
                    state,
                    amount,
                });
            }
            ActionKind::Withdrawal => {
                let amount = action.amount.ok_or(UpdateError::NoAmount)?;

                let account = self.accounts.entry(action.client_id);
                let transaction = self.transactions.entry(action.transaction_id);

                // Should be a new transaction
                if matches!(transaction, Entry::Occupied(_)) {
                    return Err(UpdateError::TransactionUsed(action.transaction_id));
                }

                // Try doing the withdrawl
                // TODO: a withdrawl from an empty account will fail due to
                // insufficient funds. Is that good enough?
                let state = match account.or_default().withdraw(amount) {
                    Ok(()) => TransactionState::Succeeded,
                    Err(e) => TransactionState::Failed(e),
                };

                // Add the transaction
                transaction.or_insert(Transaction {
                    id: action.transaction_id,
                    client: action.client_id,
                    state,
                    amount: -amount,
                });
            }
            ActionKind::Dispute => {
                let transaction = self
                    .transactions
                    .get_mut(&action.transaction_id)
                    .ok_or(UpdateError::TransactionMissing(action.transaction_id))?;

                if action.client_id != transaction.client {
                    return Err(UpdateError::ClientMismatch {
                        action: action.client_id,
                        transaction: transaction.client,
                    });
                }

                let account = self
                    .accounts
                    .get_mut(&action.client_id)
                    .ok_or(UpdateError::AccountMissing(action.client_id))?;

                // Try to hold the funds (if it was a deposit)
                // TODO: what if the transaction was a withdrawl? Is this error type sufficient?

                if transaction.amount.is_sign_positive() {
                    transaction.state = match account.hold(transaction.amount) {
                        Ok(()) => TransactionState::Disputed,
                        Err(e) => TransactionState::Failed(e),
                    };
                }
            }
            ActionKind::Resolve => {
                let transaction = self
                    .transactions
                    .get_mut(&action.transaction_id)
                    .ok_or(UpdateError::TransactionMissing(action.transaction_id))?;

                // Transaction must be disputed to be resolved
                if !matches!(transaction.state, TransactionState::Disputed) {
                    return Ok(());
                }

                if action.client_id != transaction.client {
                    return Err(UpdateError::ClientMismatch {
                        action: action.client_id,
                        transaction: transaction.client,
                    });
                }

                let account = self
                    .accounts
                    .get_mut(&action.client_id)
                    .ok_or(UpdateError::AccountMissing(action.client_id))?;

                transaction.state = match account.release(transaction.amount) {
                    Ok(()) => TransactionState::Succeeded,
                    Err(e) => TransactionState::Failed(e),
                };
            }
            ActionKind::Chargeback => {
                let transaction = self
                    .transactions
                    .get_mut(&action.transaction_id)
                    .ok_or(UpdateError::TransactionMissing(action.transaction_id))?;

                // Transaction must be disputed to be resolved
                if !matches!(transaction.state, TransactionState::Disputed) {
                    return Ok(());
                }

                if action.client_id != transaction.client {
                    return Err(UpdateError::ClientMismatch {
                        action: action.client_id,
                        transaction: transaction.client,
                    });
                }

                let account = self
                    .accounts
                    .get_mut(&action.client_id)
                    .ok_or(UpdateError::AccountMissing(action.client_id))?;

                transaction.state = match account.chargeback(transaction.amount) {
                    Ok(()) => TransactionState::Cancelled,
                    Err(e) => TransactionState::Failed(e),
                };
                account.lock();
            }
        }

        Ok(())
    }

    pub fn accounts(&self) -> AccountsIter<'_> {
        AccountsIter(self.accounts.iter())
    }

    pub fn failed_transactions(&self) -> impl Iterator<Item = &Transaction> {
        self.transactions
            .values()
            .filter(|t| matches!(t.state, TransactionState::Failed(_)))
    }
}

// Yeah, we could probably just return a vec, but where's the fun in that?
pub struct AccountsIter<'a>(std::collections::hash_map::Iter<'a, ClientId, Account>);

impl<'a> Iterator for AccountsIter<'a> {
    type Item = AccountData;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(AccountData::from)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<'a> ExactSizeIterator for AccountsIter<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error(
        "A deposit or withdrawl as requested with the same id ({0}) as an existing transaction"
    )]
    TransactionUsed(TransactionId),

    #[error(
        "An action on an existing transaction was requested but transaction {0} does not exist"
    )]
    TransactionMissing(TransactionId),

    // Technically this shouldn't happen?
    #[error("An action on an existing account was requested but account {0} does not exist")]
    AccountMissing(ClientId),

    #[error("The action and transaction it points two reference different clients (action: {action}, transaction: {transaction})")]
    ClientMismatch {
        action: ClientId,
        transaction: ClientId,
    },

    #[error("A deposit or withdrawl was requested with no amount")]
    NoAmount,
}

// TODO: should this be in the engine module? Or maybe in it's own module?
#[cfg(test)]
mod tests {
    use crate::{Action, ActionKind, ClientId, SingleThreadedEngine, SyncEngine, TransactionId};

    #[cfg(feature = "decimal")]
    use rust_decimal_macros::dec;

    // Macro for some terseness in tests
    macro_rules! action {
        ($kind:ident, $client:expr, $transaction:expr) => {
            Action {
                transaction_id: TransactionId($transaction),
                client_id: ClientId($client),
                kind: ActionKind::$kind,
                amount: None,
            }
        };
        ($kind:ident, $client:expr, $transaction:expr, $amount:expr) => {
            Action {
                transaction_id: TransactionId($transaction),
                client_id: ClientId($client),
                kind: ActionKind::$kind,

                #[cfg(feature = "decimal")]
                amount: Some(dec!($amount)),

                #[cfg(not(feature = "decimal"))]
                amount: Some($amount),
            }
        };
    }

    #[test]
    fn test_account_is_created() {
        let mut engine = SingleThreadedEngine::new();
        let _ = engine.process_all(vec![
            action!(Deposit, 1, 1, 1.5),
            action!(Withdrawal, 1, 2, 1.0),
        ]);

        let account = engine.state().accounts().next().expect("no account!");
        assert_eq!(account.total.to_string(), "0.5");
    }

    #[test]
    fn test_disputes_can_be_resolved() {
        let mut engine = SingleThreadedEngine::new();
        let _ = engine.process_all(vec![
            action!(Deposit, 1, 1, 1.5),
            action!(Dispute, 1, 1),
            action!(Resolve, 1, 1),
            action!(Withdrawal, 1, 2, 1.0),
        ]);

        let account = engine.state().accounts().next().expect("no account!");
        assert_eq!(account.total.to_string(), "0.5");
    }

    #[test]
    fn test_transactions_can_occur_after_disputes() {
        let mut engine = SingleThreadedEngine::new();
        let _ = engine.process_all(vec![
            action!(Deposit, 1, 1, 1.5),
            action!(Dispute, 1, 1),
            action!(Deposit, 1, 2, 3.0),
            action!(Withdrawal, 1, 3, 1.0),
        ]);

        let account = engine.state().accounts().next().expect("no account!");
        assert_eq!(account.available.to_string(), "2");
        assert_eq!(account.held.to_string(), "1.5");
    }

    #[test]
    fn test_chargebacks_lock_account() {
        let mut engine = SingleThreadedEngine::new();
        let _ = engine.process_all(vec![
            action!(Deposit, 1, 1, 1.5),
            action!(Dispute, 1, 1),
            action!(Chargeback, 1, 1),
            action!(Withdrawal, 1, 2, 1.0),
        ]);

        let account = engine.state().accounts().next().expect("no account!");
        assert!(account.locked);
        assert_eq!(account.total.to_string(), "0");
    }
}
