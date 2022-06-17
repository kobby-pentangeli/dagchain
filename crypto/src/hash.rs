use super::{blake::Blake, error::CryptoError};
use bytes::BytesMut;
use rand::{thread_rng, Rng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const DISPLAY_HASH_LEN: usize = 4;
const RANDOM_HASH_BUF: usize = 4096;

/// Hash representation
#[derive(Clone, Copy, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Creates a Hash from bytes
    pub fn new(data: &[u8]) -> Self {
        Self(Blake::long(&data))
    }

    /// Creates a Hash for any serializable data
    pub fn serialize<S: Serialize>(data: &S) -> Result<Self, CryptoError> {
        let s = bincode::serialize(data)
            .map_err(|e| CryptoError::SerializationError(format!("{}", e)))?;
        Ok(Self(Blake::long(&s[..])))
    }

    /// Generates a random Hash from a random buffer
    pub fn generate_random() -> Self {
        let mut bytes: [u8; RANDOM_HASH_BUF] = [0; RANDOM_HASH_BUF];
        thread_rng().fill(&mut bytes);
        Self(Blake::long(&bytes.to_vec()))
    }

    /// Converts a Hash to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Takes in byte arrays and outputs a Hash
    pub fn bytes_arrays_to_hash(bytes_arrays: Vec<Vec<u8>>) -> Self {
        let mut buf = BytesMut::new();
        for b in bytes_arrays {
            buf.copy_from_slice(&b);
        }
        Self(Blake::long(&buf[..]))
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self { 0: [0; 32] }
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::new();
        for i in 1..DISPLAY_HASH_LEN {
            s.push_str(&format!("{:02X}", self.0[i - 1]));
        }
        s.push_str("...");
        for i in (1..DISPLAY_HASH_LEN).rev() {
            s.push_str(&format!("{:02X}", self.0[32 - i]));
        }
        write!(f, "{}", s)
    }
}

impl std::fmt::Debug for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::new();
        for i in 1..DISPLAY_HASH_LEN {
            s.push_str(&format!("{:02X}", self.0[i - 1]));
        }
        s.push_str("...");
        for i in (1..DISPLAY_HASH_LEN).rev() {
            s.push_str(&format!("{:02X}", self.0[32 - i]));
        }
        write!(f, "{}", s)
    }
}

/// Hash representation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ShortHash(pub [u8; 20]);

impl ShortHash {
    /// Creates a ShortHash from bytes
    pub fn new(data: &[u8]) -> Self {
        Self(Blake::short(&data))
    }

    /// Creates a ShortHash for any serializable data
    pub fn serialize<S: Serialize>(data: &S) -> Result<Self, CryptoError> {
        let s = bincode::serialize(data)
            .map_err(|e| CryptoError::SerializationError(format!("{}", e)))?;
        Ok(Self(Blake::short(&s[..])))
    }

    /// Generates a random ShortHash from a random buffer
    pub fn generate_random() -> Self {
        let mut bytes: [u8; RANDOM_HASH_BUF] = [0; RANDOM_HASH_BUF];
        thread_rng().fill(&mut bytes);
        Self(Blake::short(&bytes.to_vec()))
    }

    /// Converts a ShortHash to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Takes in byte arrays and outputs a ShortHash
    pub fn bytes_arrays_to_hash(bytes_arrays: Vec<Vec<u8>>) -> Self {
        let mut buf = BytesMut::new();
        for b in bytes_arrays {
            buf.copy_from_slice(&b);
        }
        Self(Blake::short(&buf[..]))
    }
}

impl AsRef<[u8]> for ShortHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Default for ShortHash {
    fn default() -> Self {
        Self { 0: [0; 20] }
    }
}

/// Hash type representation.
/// Used for common Hash relations.
pub trait HashType:
    Eq
    + Ord
    + Clone
    + std::fmt::Debug
    + Send
    + Serialize
    + DeserializeOwned
    + Sync
    + std::hash::Hash
    + std::fmt::Display
    + Default
{
}

impl<N> HashType for N where
    N: Eq
        + Ord
        + Clone
        + Send
        + std::fmt::Debug
        + std::fmt::Display
        + std::hash::Hash
        + Serialize
        + DeserializeOwned
        + Sync
        + Default
{
}
