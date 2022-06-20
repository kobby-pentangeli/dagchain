use crate::{error::StorageError, Storage};
use crypto::hash::Hash;

pub struct SledStorage {
    storage: sled::Db,
    sync: bool,
}

impl Storage for SledStorage {
    /// Create new storage for DAGchain
    fn new(path: Option<&std::path::Path>) -> Result<Self, StorageError> {
        if path.is_none() {
            return Err(StorageError::NoneError.into());
        }
        Ok(SledStorage {
            storage: sled::Config::new()
                .path(path.unwrap())
                .print_profile_on_drop(false)
                .open()?,
            sync: false,
        })
    }

    /// Insert data
    fn insert(&mut self, key: Hash, value: Vec<u8>) -> Result<(), StorageError> {
        self.storage.insert(key, value)?;
        if self.sync {
            self.storage.flush()?;
        }
        Ok(())
    }

    /// Get data
    fn get(&self, key: Hash) -> Result<Vec<u8>, StorageError> {
        match self.storage.get(key)? {
            Some(data) => Ok(data.to_vec()),
            None => Err(StorageError::NoneError.into()),
        }
    }

    /// Flush data
    fn flush(&mut self) -> Result<(), StorageError> {
        self.storage.flush()?;
        Ok(())
    }
}
