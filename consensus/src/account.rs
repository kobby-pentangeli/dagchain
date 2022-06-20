use crate::{clock::Hvc, transaction::Transaction};
use crypto::hash::Hash;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Basic representation of an account
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub id: Hash,
    pub balance: u128,
    pub hvc: Hvc,
    pub last_tx_id: Hash,
    pub created: Duration,
}

/// New account choice for consensus
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AccountStateChoice {
    /// Current account state
    pub account_state_id: Hash,
    /// Choice id (usually it's transaction)
    pub tx: Transaction,
}
