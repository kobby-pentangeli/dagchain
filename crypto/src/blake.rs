use blake2b_simd::Params;
use serde::{Deserialize, Serialize};

const LONG_HASH_LEN: usize = 32;
const SHORT_HASH_LEN: usize = 20;

/// Blake hash representation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Blake;

impl Blake {
    /// Produces a long byte Hash array from source bytes
    pub fn long(src: &[u8]) -> [u8; LONG_HASH_LEN] {
        let blake_hash = Params::new()
            .hash_length(LONG_HASH_LEN)
            .to_state()
            .update(&src)
            .finalize();
        let mut hash: [u8; LONG_HASH_LEN] = [0; LONG_HASH_LEN];
        let bh = blake_hash.as_ref().to_vec();
        hash.copy_from_slice(&bh[0..LONG_HASH_LEN]);
        hash
    }

    /// Produces a short byte Hash array from source bytes
    pub fn short(src: &[u8]) -> [u8; SHORT_HASH_LEN] {
        let sh = Self::get_hash_by_len(src, SHORT_HASH_LEN);
        let mut hash: [u8; SHORT_HASH_LEN] = [0; SHORT_HASH_LEN];
        hash.copy_from_slice(&sh[0..SHORT_HASH_LEN]);
        hash
    }

    /// Retrives Blake hash by length
    pub fn get_hash_by_len(src: &[u8], hash_len: usize) -> Vec<u8> {
        Params::new()
            .hash_length(hash_len)
            .to_state()
            .update(&src)
            .finalize()
            .as_ref()
            .to_vec()
    }
}
