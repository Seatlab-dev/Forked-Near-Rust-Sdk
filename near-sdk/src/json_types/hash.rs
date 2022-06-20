use crate::CryptoHash;
use borsh::{BorshDeserialize, BorshSerialize};
use bs58::decode::Error as B58Error;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, NumberValidation, Schema, SchemaObject, StringValidation},
    JsonSchema,
};
use serde::{de, ser, Deserialize};
use serde_json::json;
use std::convert::TryFrom;

#[derive(
    Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, BorshDeserialize, BorshSerialize, Default,
)]
pub struct Base58CryptoHash(CryptoHash);

impl From<Base58CryptoHash> for CryptoHash {
    fn from(v: Base58CryptoHash) -> CryptoHash {
        v.0
    }
}

impl From<CryptoHash> for Base58CryptoHash {
    fn from(c: CryptoHash) -> Base58CryptoHash {
        Base58CryptoHash(c)
    }
}

impl ser::Serialize for Base58CryptoHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&String::from(self))
    }
}

impl<'de> de::Deserialize<'de> for Base58CryptoHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<Self>().map_err(|err| de::Error::custom(err.to_string()))
    }
}

impl From<&Base58CryptoHash> for String {
    fn from(hash: &Base58CryptoHash) -> Self {
        bs58::encode(&hash.0).into_string()
    }
}

impl TryFrom<String> for Base58CryptoHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Base58CryptoHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(value.parse()?)
    }
}

impl std::str::FromStr for Base58CryptoHash {
    type Err = ParseCryptoHashError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut crypto_hash: CryptoHash = CryptoHash::default();
        let size = bs58::decode(value).into(&mut crypto_hash)?;
        if size != std::mem::size_of::<CryptoHash>() {
            return Err(ParseCryptoHashError {
                kind: ParseCryptoHashErrorKind::InvalidLength(size),
            });
        }
        Ok(Self(crypto_hash))
    }
}

impl JsonSchema for Base58CryptoHash {
    fn is_referenceable() -> bool {
        true
    }
    fn schema_name() -> String {
        ("Base58CryptoHash").to_owned()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        // in base58 = "11111111111111111111111111111111"
        let min = Base58CryptoHash([0; 32]);
        // in base58 = "JEKNVnkbo3jma5nREBBJCDoXFVeKkD56V3xKrvRmWxFG"
        let max = Base58CryptoHash([0xff; 32]);

        let n_validation = NumberValidation {
            // maximum value that can be represented that is below u256::MAX
            maximum: Some(1.157920892373161e77f64),
            minimum: Some(0.0),
            ..Default::default()
        };

        let s_validation = StringValidation {
            max_length: Some(String::from(&max).chars().count() as u32),
            min_length: Some(String::from(&min).chars().count() as u32),
            pattern: Some(r#"^[1-9A-Za-z][^OIl]{32,44}^"#.into()),
        };

        let meta = Metadata {
            description: Some("Base58-stringfied 256-bit unsigned integer.".into()),
            default: Some(json!(String::from(&min))),
            examples: vec![json!(String::from(&min)), json!(String::from(&max))],
            ..Default::default()
        };

        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: None,
            metadata: Box::new(meta).into(),
            number: Some(Box::new(n_validation)),
            string: Some(Box::new(s_validation)),
            ..Default::default()
        }
        .into()
    }
}

#[derive(Debug)]
pub struct ParseCryptoHashError {
    kind: ParseCryptoHashErrorKind,
}

#[derive(Debug)]
enum ParseCryptoHashErrorKind {
    InvalidLength(usize),
    Base58(B58Error),
}

impl std::fmt::Display for ParseCryptoHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseCryptoHashErrorKind::InvalidLength(l) => {
                write!(f, "invalid length of the crypto hash, expected 32 got {}", l)
            }
            ParseCryptoHashErrorKind::Base58(e) => write!(f, "base58 decoding error: {}", e),
        }
    }
}

impl From<B58Error> for ParseCryptoHashError {
    fn from(e: B58Error) -> Self {
        Self { kind: ParseCryptoHashErrorKind::Base58(e) }
    }
}

impl std::error::Error for ParseCryptoHashError {}
