use serde::{Deserialize, Serialize};

mod account;
mod action;
mod engine;
mod state;
mod transaction;

pub use account::{Account, AccountData, AccountError};
pub use action::{Action, ActionKind};
pub use engine::{MultiThreadedEngine, SingleThreadedEngine, SyncEngine};
pub use transaction::{Transaction, TransactionState};

#[cfg(feature = "decimal")]
type Amount = rust_decimal::Decimal;

#[cfg(not(feature = "decimal"))]
type Amount = f64;

/// Newtype'd client id, so it can never be mixed up with `TransactionId`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct ClientId(pub(crate) u16);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype'd transaction id, so it can never be mixed up with `ClientId`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct TransactionId(pub(crate) u32);

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
