use crate::node::event::Event;
use thiserror::Error;

/// P2p-related errors
#[derive(Debug, Error)]
pub enum P2pError {
    #[error("Cryptography error: {0}")]
    CryptoError(crypto::error::CryptoError),
    #[error("Bincode (De)Serialization error: {0}")]
    BincodeError(bincode::Error),
    #[error("Multibase encode/decode error: {0}")]
    MultibaseError(multibase::Error),
    #[error("Quic error: {0}")]
    QuicP2pError(quic_p2p::QuicP2pError),
    #[error("I/O error: {0}")]
    IoError(std::io::Error),
    #[error("Crossbeam receiver error: {0}")]
    CrossbeamReceiverError(crossbeam_channel::RecvError),
    #[error("Crossbeam sender error: {0}")]
    CrossbeamSenderError(crossbeam_channel::SendError<Event>),
    #[error("Invalid signature error")]
    InvalidSignature,
    #[error("Custom error: {0}")]
    CustomError(String),
}
