use crate::{account::Account, clock::Hvc};
use crypto::{
    hash::Hash,
    signature::{PrivateKey, PublicKey, Signature},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Transaction type
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum TransactionType {
    CreateAccount,
    Transfer,
}

/// Transaction status
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum TransactionStatus {
    None,
    Pending,
    Accepted,
    Rejected,
}

/// Basic representation of a transaction
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Transaction {
    id: Option<Hash>,
    pub parent: Hash,
    pub origin: Hash,
    pub destination: Hash,
    pub amount: u128,
    pub status: TransactionStatus,
    pub tx_type: TransactionType,
    pub payload: Vec<u8>,
    pub hvc: Hvc,
    pub timestamp: Duration,
    signatures: HashMap<Hash, Signature>,
    agg_signature: Option<Signature>,
    children: Vec<Hash>,
}
