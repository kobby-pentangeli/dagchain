//! # Crypto errors

use bls_signatures::Error as BlsError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("BLS Signature error: {0}")]
    BlsSignatureError(BlsError),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Option(None) returned error")]
    NoneError,
}
