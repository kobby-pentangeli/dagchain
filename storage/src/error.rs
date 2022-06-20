use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Sled Error: {0}")]
    SledError(sled::Error),

    #[error("Option<None>: an error!")]
    NoneError,
}

impl From<sled::Error> for StorageError {
    #[inline]
    fn from(e: sled::Error) -> Self {
        StorageError::SledError(e)
    }
}
