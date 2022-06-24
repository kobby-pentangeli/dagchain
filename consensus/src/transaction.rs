use crate::{account::Account, clock::Hvc};
use crypto::{
    error::CryptoError,
    hash::Hash,
    signature::{PrivateKey, PublicKey, Signature},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

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

impl Transaction {
    pub fn new(
        parent: Hash,
        origin: Account,
        destination: Hash,
        amount: u128,
        tx_type: TransactionType,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id: None,
            parent,
            origin: origin.id,
            destination,
            amount,
            status: TransactionStatus::Pending,
            tx_type,
            payload,
            hvc: Hvc::new(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            signatures: HashMap::new(),
            agg_signature: None,
            children: vec![],
        }
    }

    /// Same as the Transation::new()
    /// But sets the timestamp to `0` so all nodes can create the genesis transaction
    pub fn genesis(
        parent: Hash,
        origin: Account,
        destination: Hash,
        amount: u128,
        tx_type: TransactionType,
        payload: Vec<u8>,
    ) -> Self {
        let mut tx = Self::new(parent, origin, destination, amount, tx_type, payload);
        tx.timestamp = Duration::from_secs(0);
        tx
    }

    /// Apply transaction changes for Account
    pub fn apply(&self, origin: &mut Account, destination: &mut Account) {
        origin
            .update_last_tx(&self.id.unwrap())
            .update_hvc()
            .decrease_balance(self.amount);
        destination
            .update_last_tx(&self.id.unwrap())
            .update_hvc()
            .increase_balance(self.amount);
    }

    /// Get restricted version of transaction for:
    /// * Hashing
    /// * Verifying
    /// * Signing
    fn restricted_tx(&self) -> Self {
        let mut tx = self.clone();
        tx.id = None;
        tx.signatures = HashMap::new();
        tx.agg_signature = None;
        tx.children = vec![];
        tx
    }

    /// Calculate ID of transaction
    pub fn calculate_tx_id(&mut self) -> Result<&mut Self, CryptoError> {
        let tx = self.restricted_tx();
        self.id = Some(Hash::serialize(&tx)?);
        Ok(self)
    }

    /// Sign transaction
    pub fn sign_tx(&self, private_key: &PrivateKey) -> Result<Signature, CryptoError> {
        let tx = self.restricted_tx();
        let payload = bincode::serialize(&tx)
            .map_err(|e| CryptoError::SerializationError(format!("{}", e)))?;
        Ok(Signature::sign(private_key, payload))
    }

    /// Add tx signature to list of signatures
    pub fn set_signature(&mut self, pubkey: &PublicKey, sig: &Signature) -> &mut Self {
        self.signatures.insert(Hash::new(&pubkey.to_bytes()), *sig);
        self
    }

    /// Sign tx and add signature to list of signatures
    pub fn sign_and_set_signature(
        &mut self,
        private_key: &PrivateKey,
    ) -> Result<&mut Self, CryptoError> {
        let sig = self.sign_tx(private_key)?;
        Ok(self.set_signature(&private_key.public_key(), &sig))
    }

    /// Aggregate tx signatures
    pub fn aggregate_signatures(&mut self) -> Result<&mut Self, CryptoError> {
        let mut sigs: Vec<Signature> = vec![];
        for (_, sig) in self.signatures.iter() {
            sigs.push(*sig);
        }
        self.agg_signature =
            Some(Signature::aggregate(&sigs).map_err(|e| CryptoError::BlsSignatureError(e))?);
        Ok(self)
    }

    /// Accept transaction
    pub fn accept_tx(&mut self, private_key: &PrivateKey) -> Result<&mut Self, CryptoError> {
        self.set_tx_status(TransactionStatus::Accepted)
            .sign_and_set_signature(&private_key)?
            .aggregate_signatures()?;
        Ok(self)
    }

    /// Set transaction status
    pub fn set_tx_status(&mut self, status: TransactionStatus) -> &mut Self {
        self.status = status;
        self
    }

    /// Retrieve all signatures
    pub fn get_sigs(&self) -> HashMap<Hash, Signature> {
        self.signatures.clone()
    }

    /// Retrieve aggregated signature
    pub fn get_aggregate_sig(&self) -> Option<Signature> {
        self.agg_signature
    }

    /// Verify the signature of a transaction
    pub fn verify_tx_sig(&mut self, pubkey: &PublicKey) -> Result<bool, CryptoError> {
        let sig = self.signatures.get(&Hash::new(&pubkey.to_bytes()));
        if sig.is_none() {
            return Ok(false);
        }
        let tx = self.restricted_tx();
        let payload =
            bincode::serialize(&tx).map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        Ok(sig.unwrap().verify(&pubkey, payload))
    }

    pub fn get_tx_id(&self) -> Hash {
        self.id.unwrap()
    }

    pub fn set_tx_id(&mut self, id: Hash) {
        self.id = Some(id);
    }

    pub fn set_hvc(&mut self, source: &Account) -> &mut Self {
        self.hvc = source.hvc.clone();
        self
    }

    pub fn set_children(&mut self, children: Vec<Hash>) -> &mut Self {
        self.children = children;
        self
    }

    pub fn get_children(&self) -> Vec<Hash> {
        self.children.clone()
    }

    pub fn check_transfer_availability(&self, source: &Account) -> bool {
        source.balance >= self.amount
    }
}

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
