use serde::Deserialize;

use crate::{Amount, ClientId, TransactionId};

/// An individual input item, representing an action on a transaction
#[derive(Debug, Deserialize)]
pub struct Action {
    #[serde(rename = "tx")]
    pub transaction_id: TransactionId,

    #[serde(rename = "client")]
    pub client_id: ClientId,

    /// Could be `r#type`, but typing (ha) that can be tedious and we've already
    /// lost some semantics of the original name.
    #[serde(rename = "type")]
    pub kind: ActionKind,

    pub amount: Option<Amount>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionKind {
    /// Add funds to an account, creating it if it doesn't exist
    Deposit,

    /// Withdraw the funds (if available) from a client's account
    Withdrawal,

    /// Dispute an existing transaction, holding the
    Dispute,
    Resolve,
    Chargeback,
}
