/// BLS Signature
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Signature(bls_signatures::Signature);

impl Signature {
    /// Initialize a Signature
    pub fn new(s: bls_signatures::Signature) -> Self {
        Self(s)
    }

    /// Sign a message
    pub fn sign<T>(private_key: &PrivateKey, data: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self::new(private_key.0.sign(data))
    }

    /// Retrieve a Signature from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, bls_signatures::Error> {
        use bls_signatures::Serialize;
        Ok(Self(bls_signatures::Signature::from_bytes(data)?))
    }

    /// Convert a Signature into a byte array
    pub fn as_bytes(&self) -> Vec<u8> {
        use bls_signatures::Serialize;
        self.0.as_bytes()
    }

    /// Verify a signed message
    pub fn verify<T>(&self, pub_key: &PublicKey, data: T) -> bool
    where
        T: AsRef<[u8]>,
    {
        pub_key.0.verify(self.0, data)
    }

    /// Aggregate Signatures
    pub fn aggregate(sigs: &[Self]) -> Result<Self, bls_signatures::Error> {
        use bls_signatures::Signature as BlsSignature;
        let mut signatures: Vec<BlsSignature> = vec![];
        sigs.iter().map(|x| x.0).for_each(|x| signatures.push(x));

        let aggr_sig = bls_signatures::aggregate(&signatures)?;
        Ok(Self::new(aggr_sig))
    }
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use bls_signatures::Serialize;
        serializer.serialize_bytes(&self.0.as_bytes())
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(SignatureVisitor)
    }
}

/// BLS Public Key
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PublicKey(bls_signatures::PublicKey);

impl PublicKey {
    /// Generate PublicKey from bytes
    pub fn from_bytes(raw: &[u8]) -> Result<Self, bls_signatures::Error> {
        use bls_signatures::Serialize;
        Ok(Self(bls_signatures::PublicKey::from_bytes(raw)?))
    }

    /// Convert PublicKey to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        use bls_signatures::Serialize;
        self.0.as_bytes()
    }
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use bls_signatures::Serialize;
        serializer.serialize_bytes(&self.0.as_bytes())
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PublicKeyVisitor)
    }
}

/// BLS Private Key
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrivateKey(bls_signatures::PrivateKey);

impl PrivateKey {
    /// Generate PrivateKey from bytes
    pub fn from_bytes(raw: &[u8]) -> Result<Self, bls_signatures::Error> {
        use bls_signatures::Serialize;
        Ok(Self(bls_signatures::PrivateKey::from_bytes(raw)?))
    }

    /// Convert PrivateKey to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        use bls_signatures::Serialize;
        self.0.as_bytes()
    }

    /// Generate a random PrivateKey
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let pk = bls_signatures::PrivateKey::generate(&mut rng);
        Self(pk)
    }

    /// Retrieve the PublicKey for this PrivateKey
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.public_key())
    }
}

impl serde::Serialize for PrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use bls_signatures::Serialize;
        serializer.serialize_bytes(&self.0.as_bytes())
    }
}

impl<'de> serde::Deserialize<'de> for PrivateKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PrivateKeyVisitor)
    }
}

struct SignatureVisitor;

impl<'de> serde::de::Visitor<'de> for SignatureVisitor {
    type Value = Signature;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a signature byte array")
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Signature::from_bytes(v).unwrap())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Signature::from_bytes(v.as_bytes()).unwrap())
    }
}

struct PublicKeyVisitor;

impl<'de> serde::de::Visitor<'de> for PublicKeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a signature byte array")
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PublicKey::from_bytes(v).unwrap())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PublicKey::from_bytes(v.as_bytes()).unwrap())
    }
}

struct PrivateKeyVisitor;

impl<'de> serde::de::Visitor<'de> for PrivateKeyVisitor {
    type Value = PrivateKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a signature byte array")
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PrivateKey::from_bytes(v).unwrap())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PrivateKey::from_bytes(v.as_bytes()).unwrap())
    }
}

#[test]
fn test_signature() {
    let secret_key = PrivateKey::generate();
    let data = "data to be signed";
    let signature = Signature::sign(&secret_key, data);
    let public_key = secret_key.public_key();
    assert!(signature.verify(&public_key, data));

    let serialized_sig = bincode::serialize(&signature);
    assert!(serialized_sig.is_ok());

    let deserialized_sig = bincode::deserialize::<Signature>(&serialized_sig.unwrap()[..]);
    assert!(deserialized_sig.is_ok());

    let s_sig = deserialized_sig.unwrap();
    assert_eq!(signature, s_sig);
    assert!(s_sig.verify(&public_key, data));
}

#[test]
fn test_public_key() {
    let secret_key = PrivateKey::generate();
    let public_key = secret_key.public_key();

    let serialized_public_key = bincode::serialize(&public_key);
    assert!(serialized_public_key.is_ok());

    let deserialized_public_key =
        bincode::deserialize::<PublicKey>(&serialized_public_key.unwrap()[..]);
    assert!(deserialized_public_key.is_ok());

    let s_pub_key = deserialized_public_key.unwrap();
    assert_eq!(public_key, s_pub_key);
}

#[test]
fn test_private_key() {
    let secret_key = PrivateKey::generate();
    let serialized_secret_key = bincode::serialize(&secret_key);
    assert!(serialized_secret_key.is_ok());

    let deserialized_secret_key =
        bincode::deserialize::<PrivateKey>(&serialized_secret_key.unwrap()[..]);
    assert!(deserialized_secret_key.is_ok());

    let s_secret = deserialized_secret_key.unwrap();
    assert_eq!(secret_key, s_secret);
}
