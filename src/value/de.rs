use super::Value;
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "any valid SION value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            fn visit_i8<E>(self, value: i8) -> Result<Value, E> {
                Ok(Value::Int(i64::from(value)))
            }

            fn visit_i16<E>(self, value: i16) -> Result<Value, E> {
                Ok(Value::Int(i64::from(value)))
            }

            fn visit_i32<E>(self, value: i32) -> Result<Value, E> {
                Ok(Value::Int(i64::from(value)))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Int(i64::from(value)))
            }

            fn visit_u8<E>(self, value: u8) -> Result<Value, E> {
                Ok(Value::Int(i64::from(value)))
            }

            fn visit_u16<E>(self, value: u16) -> Result<Value, E> {
                Ok(Value::Int(i64::from(value)))
            }

            fn visit_u32<E>(self, value: u32) -> Result<Value, E> {
                Ok(Value::Int(i64::from(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                // TODO: return Err
                Ok(Value::Int(value as i64))
            }

            fn visit_f32<E>(self, value: f32) -> Result<Value, E> {
                Ok(Value::Double(f64::from(value)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::Double(f64::from(value)))
            }

            fn visit_char<E>(self, value: char) -> Result<Value, E> {
                Ok(Value::String(value.to_string()))
            }

            fn visit_str<E>(self, value: &str) -> Result<Value, E> {
                Ok(Value::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Value, E> {
                let mut v = vec![];
                v.extend_from_slice(value);
                Ok(Value::Data(v))
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Value, E> {
                Ok(Value::Data(value))
            }

            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut contents = vec![];
                while let Some(v) = seq.next_element()? {
                    contents.push(v);
                }
                Ok(Value::Array(::sequence::Array { contents }))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut contents = vec![];
                while let Some(item) = map.next_entry()? {
                    contents.push(item);
                }
                Ok(Value::Map(::sequence::Map { contents }))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
