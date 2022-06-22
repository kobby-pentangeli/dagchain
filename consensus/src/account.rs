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

impl Account {
    /// Create account
    pub fn create(account_id: &Hash, tx_id: &Hash) -> Self {
        Self {
            id: *account_id,
            balance: 0,
            hvc: Hvc::new(),
            last_tx_id: *tx_id,
            created: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
        }
    }

    /// Increase account balance
    pub fn increase_balance(&mut self, balance: u128) -> &mut Self {
        self.balance += balance;
        self
    }

    /// Decrease account balance
    pub fn decrease_balance(&mut self, balance: u128) -> &mut Self {
        self.balance -= balance;
        self
    }

    /// Update last transaction ID
    pub fn update_last_tx(&mut self, tx_id: &Hash) -> &mut Self {
        self.last_tx_id = *tx_id;
        self
    }

    /// Update HVC
    pub fn update_hvc(&mut self) -> &mut Self {
        self.hvc.order().increment();
        self
    }

    /// Get current HVC value
    pub fn get_hvc(&mut self) -> u64 {
        self.hvc.order().get()
    }
}

/// New account choice for consensus
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AccountStateChoice {
    /// Current account state
    pub account_state_id: Hash,
    /// Choice id (usually it's transaction)
    pub tx: Transaction,
}

impl AccountStateChoice {
    pub fn new(account_state_id: Hash, tx: Transaction) -> Self {
        Self {
            account_state_id,
            tx,
        }
    }
}
