use crate::error::P2pError;
use crypto::{
    hash::Hash,
    signature::{PrivateKey, PublicKey, Signature},
};
pub use public_id::PublicId;

mod public_id;

/// Identity of a p2p node
#[derive(Clone)]
pub struct Identity {
    private_key: PrivateKey,
    public_key: PublicKey,
}

impl Identity {
    pub fn new() -> Self {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        Self {
            private_key,
            public_key,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> Signature {
        Signature::sign(&self.private_key, message)
    }

    pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> Result<(), P2pError> {
        if signature.verify(&self.public_key, message) {
            Ok(())
        } else {
            Err(P2pError::InvalidSignature)
        }
    }

    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn get_private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    pub fn get_public_id(&self) -> PublicId {
        PublicId {
            public_key: self.public_key,
        }
    }

    pub fn get_our_hash(&self) -> Result<Hash, P2pError> {
        Ok(Hash::serialize(&self.public_key).map_err(|e| P2pError::CryptoError(e))?)
    }

    pub fn decode(encoded_id: &str) -> Result<Self, P2pError> {
        let (_base, bytes) =
            multibase::decode(encoded_id).map_err(|e| P2pError::MultibaseError(e))?;
        Ok(bincode::deserialize(&bytes).map_err(|e| P2pError::BincodeError(e))?)
    }

    pub fn encode(&self) -> Result<String, P2pError> {
        let buffer = bincode::serialize(self).map_err(|e| P2pError::BincodeError(e))?;
        Ok(multibase::encode(multibase::Base::Base32Z, buffer))
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}

impl serde::Serialize for Identity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&self.private_key, &self.public_key).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Identity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (private_key, public_key) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Identity {
            private_key,
            public_key,
        })
    }
}

#[test]
fn test_encode_decode_identity() {
    let identity = Identity::new();
    let encoded_id = identity.encode().unwrap();
    let recovered_id = Identity::decode(&encoded_id).unwrap();
    assert_eq!(identity.get_private_key(), recovered_id.get_private_key());
}

#[test]
fn test_serialize_deserialize_public_id() {
    let identity = Identity::new();
    let bytes = bincode::serialize(&identity.get_public_id()).unwrap();
    let public_id: PublicId = bincode::deserialize(&bytes).unwrap();
    assert_eq!(identity.get_public_id().public_key, public_id.public_key);
}

#[test]
fn test_signing_and_verification() {
    let id1 = Identity::new();
    let id2 = Identity::new();

    let message = vec![1, 2, 3, 4, 5];
    let signature = id1.sign_message(&message);
    assert!(id1.verify_signature(&message, &signature).is_ok());

    let invalid_sig_res = id2.verify_signature(&message, &signature);
    assert!(matches!(invalid_sig_res, Err(P2pError::InvalidSignature)));
}
