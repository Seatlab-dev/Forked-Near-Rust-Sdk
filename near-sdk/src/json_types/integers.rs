//! Helper classes to serialize and deserialize large integer types into base-10 string
//! representations.
//! NOTE: JSON standard can only work with integer up to 53 bits. So we need helper classes for
//! 64-bit and 128-bit integers.

use borsh::{BorshDeserialize, BorshSerialize};
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Metadata, NumberValidation, Schema, SchemaObject, StringValidation},
    JsonSchema,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

macro_rules! impl_str_type {
    ($iden: ident, $ty: tt) => {
        #[derive(Debug, Clone, Copy, PartialEq, BorshDeserialize, BorshSerialize)]
        pub struct $iden(pub $ty);

        impl From<$ty> for $iden {
            fn from(v: $ty) -> Self {
                Self(v)
            }
        }

        impl From<$iden> for $ty {
            fn from(v: $iden) -> $ty {
                v.0
            }
        }

        impl Serialize for $iden {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $iden {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                let s: String = Deserialize::deserialize(deserializer)?;
                Ok(Self(
                    str::parse::<$ty>(&s)
                        .map_err(|err| serde::de::Error::custom(err.to_string()))?,
                ))
            }
        }
    };
}

impl_str_type!(U128, u128);
impl_str_type!(U64, u64);
impl_str_type!(I128, i128);
impl_str_type!(I64, i64);

impl JsonSchema for U128 {
    fn is_referenceable() -> bool {
        true
    }
    fn schema_name() -> String {
        ("U128").to_owned()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let n_validation = NumberValidation {
            // maximum value that can be represented that is below u128::MAX
            maximum: Some(3.402823669209384e38f64),
            minimum: Some(0.0),
            ..Default::default()
        };

        let s_validation = StringValidation {
            max_length: Some(u128::MAX.to_string().chars().count() as u32),
            min_length: Some(u128::MIN.to_string().chars().count() as u32),
            // 340282366920938463463374607431768211455
            pattern: Some(r#"^[0-9]{1,39}$"#.into()),
        };

        let meta = Metadata {
            description: Some("Stringfied 128-bit unsigned integer.".into()),
            default: Some(json!(u128::MIN.to_string())),
            examples: vec![json!(u128::MIN.to_string()), json!(u128::MAX.to_string())],
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

impl JsonSchema for U64 {
    fn is_referenceable() -> bool {
        true
    }
    fn schema_name() -> String {
        ("U64").to_owned()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let n_validation = NumberValidation {
            // maximum value that can be represented that is below u64::MAX
            maximum: Some(18446744073709550000.),
            minimum: Some(0.0),
            ..Default::default()
        };

        let s_validation = StringValidation {
            max_length: Some(u64::MAX.to_string().chars().count() as u32),
            min_length: Some(u64::MIN.to_string().chars().count() as u32),
            // 18446744073709551615
            pattern: Some(r#"^[0-9]{1,20}$"#.into()),
        };

        let meta = Metadata {
            description: Some("Stringfied 64-bit unsigned integer.".into()),
            default: Some(json!(u64::MIN.to_string())),
            examples: vec![json!(u64::MIN.to_string()), json!(u64::MAX.to_string())],
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

impl JsonSchema for I128 {
    fn is_referenceable() -> bool {
        true
    }
    fn schema_name() -> String {
        ("I128").to_owned()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let n_validation = NumberValidation {
            // maximum value that can be represented that is below i128::MAX
            maximum: Some(1.7014118346046923e38),
            // minimum value that can be represented that is above i128::MIN
            minimum: Some(-1.7014118346046923e38),
            ..Default::default()
        };

        let s_validation = StringValidation {
            max_length: Some(
                (i128::MAX.to_string().chars().count()).max(i128::MIN.to_string().chars().count())
                    as u32,
            ),
            min_length: Some(0i128.to_string().chars().count() as u32),
            // -170141183460469231731687303715884105728
            pattern: Some(r#"^-?[0-9]{1,39}$"#.into()),
        };

        let meta = Metadata {
            description: Some("Stringfied 128-bit signed integer.".into()),
            default: Some(json!(0i128.to_string())),
            examples: vec![
                json!(0i128.to_string()),
                json!(i128::MIN.to_string()),
                json!(i128::MAX.to_string()),
            ],
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

impl JsonSchema for I64 {
    fn is_referenceable() -> bool {
        true
    }
    fn schema_name() -> String {
        ("I64").to_owned()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        let n_validation = NumberValidation {
            // maximum value that can be represented that is below i64::MAX
            maximum: Some(9223372036854775000.),
            // minimum value that can be represented that is above i64::MIN
            minimum: Some(-9223372036854775000.),
            ..Default::default()
        };

        let s_validation = StringValidation {
            max_length: Some(
                (i64::MAX.to_string().chars().count()).max(i64::MIN.to_string().chars().count())
                    as u32,
            ),
            min_length: Some(0i64.to_string().chars().count() as u32),
            // -9223372036854775808
            pattern: Some(r#"^-?[0-9]{1,19}$"#.into()),
        };

        let meta = Metadata {
            description: Some("Stringfied 64-bit signed integer.".into()),
            default: Some(json!(0i64.to_string())),
            examples: vec![
                json!(0i64.to_string()),
                json!(i64::MIN.to_string()),
                json!(i64::MAX.to_string()),
            ],
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

    macro_rules! test_serde {
        ($str_type: tt, $int_type: tt, $number: expr) => {
            let a: $int_type = $number;
            let str_a: $str_type = a.into();
            let b: $int_type = str_a.into();
            assert_eq!(a, b);

            let str: String = serde_json::to_string(&str_a).unwrap();
            let deser_a: $str_type = serde_json::from_str(&str).unwrap();
            assert_eq!(a, deser_a.0);
        };
    }

    #[test]
    fn test_u128() {
        test_serde!(U128, u128, 0);
        test_serde!(U128, u128, 1);
        test_serde!(U128, u128, 123);
        test_serde!(U128, u128, 10u128.pow(18));
        test_serde!(U128, u128, 2u128.pow(100));
        test_serde!(U128, u128, u128::max_value());
    }

    #[test]
    fn test_i128() {
        test_serde!(I128, i128, 0);
        test_serde!(I128, i128, 1);
        test_serde!(I128, i128, -1);
        test_serde!(I128, i128, 123);
        test_serde!(I128, i128, 10i128.pow(18));
        test_serde!(I128, i128, 2i128.pow(100));
        test_serde!(I128, i128, -(2i128.pow(100)));
        test_serde!(I128, i128, i128::max_value());
        test_serde!(I128, i128, i128::min_value());
    }

    #[test]
    fn test_u64() {
        test_serde!(U64, u64, 0);
        test_serde!(U64, u64, 1);
        test_serde!(U64, u64, 123);
        test_serde!(U64, u64, 10u64.pow(18));
        test_serde!(U64, u64, 2u64.pow(60));
        test_serde!(U64, u64, u64::max_value());
    }

    #[test]
    fn test_i64() {
        test_serde!(I64, i64, 0);
        test_serde!(I64, i64, 1);
        test_serde!(I64, i64, -1);
        test_serde!(I64, i64, 123);
        test_serde!(I64, i64, 10i64.pow(18));
        test_serde!(I64, i64, 2i64.pow(60));
        test_serde!(I64, i64, -(2i64.pow(60)));
        test_serde!(I64, i64, i64::max_value());
        test_serde!(I64, i64, i64::min_value());
    }
}
