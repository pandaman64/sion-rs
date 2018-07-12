use super::Value;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use self::Value::*;
        match self {
            Nil => serializer.serialize_none(),
            Bool(b) => serializer.serialize_bool(*b),
            Int(i) => serializer.serialize_i64(*i),
            Double(f) => serializer.serialize_f64(*f),
            String(s) => serializer.serialize_str(s),
            Data(v) => serializer.serialize_bytes(v),
            Date(_) => unimplemented!(),
            Array(::sequence::Array { contents: v }) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for x in v.iter() {
                    seq.serialize_element(x)?;
                }
                seq.end()
            }
            Map(::sequence::Map { contents: m }) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (key, value) in m.iter() {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}
