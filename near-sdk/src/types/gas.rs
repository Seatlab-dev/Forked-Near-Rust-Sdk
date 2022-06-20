use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use core::ops;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, NumberValidation, Schema, SchemaObject, StringValidation},
    JsonSchema,
};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

/// Represents the amount of NEAR tokens in "gas units" which are used to fund transactions.
#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    Hash,
    BorshSchema,
)]
#[repr(transparent)]
pub struct Gas(pub u64);

impl Gas {
    /// One Tera gas, which is 10^12 gas units.
    pub const ONE_TERA: Gas = Gas(1_000_000_000_000);
}

impl Serialize for Gas {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = [0u8; 20];
        let remainder = {
            use std::io::Write;

            let mut w: &mut [u8] = &mut buf;
            write!(w, "{}", self.0).unwrap_or_else(|_| crate::env::abort());
            w.len()
        };
        let len = buf.len() - remainder;

        let s = std::str::from_utf8(&buf[..len]).unwrap_or_else(|_| crate::env::abort());
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for Gas {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        s.parse::<u64>().map(Self).map_err(|err| de::Error::custom(err.to_string()))
    }
}

impl From<u64> for Gas {
    fn from(amount: u64) -> Self {
        Self(amount)
    }
}

impl From<Gas> for u64 {
    fn from(gas: Gas) -> Self {
        gas.0
    }
}

impl ops::Add for Gas {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl ops::AddAssign for Gas {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl ops::SubAssign for Gas {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl ops::Sub for Gas {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl ops::Mul<u64> for Gas {
    type Output = Self;

    fn mul(self, other: u64) -> Self {
        Self(self.0 * other)
    }
}

impl ops::Div<u64> for Gas {
    type Output = Self;

    fn div(self, other: u64) -> Self {
        Self(self.0 / other)
    }
}

impl ops::Rem<u64> for Gas {
    type Output = Self;

    fn rem(self, rhs: u64) -> Self::Output {
        Self(self.0.rem(rhs))
    }
}

impl JsonSchema for Gas {
    fn is_referenceable() -> bool {
        true
    }
    fn schema_name() -> String {
        ("Gas").to_owned()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let max = 300_000_000_000_000.;
        let n_validation = NumberValidation {
            // maximum value that can be represented that is below u64::MAX,
            // and below the maximum allowed gas for a function call
            //
            // ref: https://docs.near.org/docs/concepts/gas
            maximum: Some(max),
            minimum: Some(0.0),
            ..Default::default()
        };

        let s_validation = StringValidation {
            max_length: Some(max.to_string().chars().count() as u32),
            min_length: Some(u64::MIN.to_string().chars().count() as u32),
            // 300000000000000
            pattern: Some(r#"^[0-9]{1,15}$"#.into()),
        };

        let meta = Metadata {
            description: Some(r#"Stringfied 64-bit unsigned integer. Represents the amount of NEAR tokens in "gas units" which are used to fund transactions. See [docs/gas](https://docs.near.org/docs/concepts/gas) for more info."#.into()),
            default: Some(json!(u64::MIN.to_string())),
            examples: vec![json!(u64::MIN.to_string()), json!(max.to_string())],
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_json_ser(val: u64) {
        let gas = Gas(val);
        let ser = serde_json::to_string(&gas).unwrap();
        assert_eq!(ser, format!("\"{}\"", val));
        let de: Gas = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.0, val);
    }

    #[test]
    fn json_ser() {
        test_json_ser(u64::MAX);
        test_json_ser(8);
        test_json_ser(0);
    }
}
