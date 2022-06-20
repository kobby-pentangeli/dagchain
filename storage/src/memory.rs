use crate::{error::StorageError, Storage};
use crypto::hash::Hash;
use std::collections::HashMap;

pub struct MemoryStorage {
    storage: HashMap<Hash, Vec<u8>>,
}

impl Storage for MemoryStorage {
    /// Create new storage for DAGchain
    fn new(_p: Option<&std::path::Path>) -> Result<Self, StorageError> {
        Ok(MemoryStorage {
            storage: HashMap::new(),
        })
    }

    /// Insert data
    fn insert(&mut self, key: Hash, value: Vec<u8>) -> Result<(), StorageError> {
        self.storage.insert(key, value);
        Ok(())
    }

    /// Get data
    fn get(&self, key: Hash) -> Result<Vec<u8>, StorageError> {
        match self.storage.get(&key) {
            Some(data) => Ok(data.to_vec()),
            None => Err(StorageError::NoneError.into()),
        }
    }

    /// Flush data
    fn flush(&mut self) -> Result<(), StorageError> {
        Ok(())
    }
}
