//! This module owns duplicate-refusing repository JSON admission.

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

pub(super) fn parse(raw: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str::<UniqueValue>(raw).map(|value| value.0)
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<DeserializerType>(
        deserializer: DeserializerType,
    ) -> Result<Self, DeserializerType::Error>
    where
        DeserializerType: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueVisitor)
    }
}

struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON without duplicate object members")
    }

    fn visit_bool<ErrorType>(self, value: bool) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<ErrorType>(self, value: i64) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<ErrorType>(self, value: u64) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Ok(UniqueValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<ErrorType>(self, value: f64) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(UniqueValue)
            .ok_or_else(|| ErrorType::custom("JSON number is not finite"))
    }

    fn visit_str<ErrorType>(self, value: &str) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<ErrorType>(self, value: String) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_none<ErrorType>(self) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_unit<ErrorType>(self) -> Result<Self::Value, ErrorType>
    where
        ErrorType: de::Error,
    {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_some<DeserializerType>(
        self,
        deserializer: DeserializerType,
    ) -> Result<Self::Value, DeserializerType::Error>
    where
        DeserializerType: Deserializer<'de>,
    {
        UniqueValue::deserialize(deserializer)
    }

    fn visit_seq<Sequence>(self, mut sequence: Sequence) -> Result<Self::Value, Sequence::Error>
    where
        Sequence: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_map<Object>(self, mut object: Object) -> Result<Self::Value, Object::Error>
    where
        Object: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate object member {key:?}"
                )));
            }
            let value = object.next_value::<UniqueValue>()?;
            values.insert(key, value.0);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}
