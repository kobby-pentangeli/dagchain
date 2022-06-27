use crypto::signature::PublicKey;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// Public identity of a p2p node
#[derive(Clone, Copy)]
pub struct PublicId {
    pub public_key: PublicKey,
}

impl std::fmt::Debug for PublicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublicId")
            .field("public_key", &self.public_key)
            .finish()
    }
}

impl PartialEq for PublicId {
    fn eq(&self, other: &Self) -> bool {
        self.public_key == other.public_key
    }
}

impl Eq for PublicId {}

impl Serialize for PublicId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&self.public_key).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PublicId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let public_key = Deserialize::deserialize(deserializer)?;
        Ok(PublicId { public_key })
    }
}

#[cfg(test)]
mod test {
    use super::super::Identity;
    use super::PublicId;

    #[test]
    fn test_serialize_and_deserialize_public_id() {
        let identity = Identity::new();
        let bytes = bincode::serialize(&identity.get_public_id()).unwrap();
        let public_id: PublicId = bincode::deserialize(&bytes).unwrap();
        assert_eq!(identity.get_public_id().public_key, public_id.public_key);
    }
}
