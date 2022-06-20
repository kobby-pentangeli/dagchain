#![warn(clippy::all)]

use crypto::hash::Hash;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod memory;
pub mod sled;

pub use error::StorageError;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum StorageType {
    Memory,
    Sled,
}

pub trait Storage: Send + Sync {
    /// Create new storage
    fn new(path: Option<&std::path::Path>) -> Result<Self, StorageError>
    where
        Self: std::marker::Sized;

    /// Insert data
    fn insert(&mut self, key: Hash, value: Vec<u8>) -> Result<(), StorageError>;

    /// Get data
    fn get(&self, key: Hash) -> Result<Vec<u8>, StorageError>;

    /// Flush data
    fn flush(&mut self) -> Result<(), StorageError>;
}
