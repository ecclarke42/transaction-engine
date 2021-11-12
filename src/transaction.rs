use crate::{AccountError, Amount, ClientId, TransactionId};

/// An individual transaction, deserialized from the input csv.
///
/// Note: This could be a terser type by moving all the serde attributes to a
/// intermediate deserializer class (particularly if we had to support multiple
/// input formats and normalize them to a `Transaction` model), but that seems
/// like overkill for this exercise.
#[derive(Debug)]
pub struct Transaction {
    pub id: TransactionId,
    pub client: ClientId,

    pub state: TransactionState,

    pub amount: Amount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Succeeded,
    Failed(AccountError),

    Disputed,
    Cancelled,
}
