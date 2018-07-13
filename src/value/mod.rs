pub mod de;
pub mod ser;
use sequence::{Array, Map};

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Data(Vec<u8>),
    Date(f64),
    Array(Array),
    Map(Map),
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_serialize_simple() {
        use super::Value::*;
        use to_string;

        assert_eq!(to_string(&Nil).unwrap(), "nil".to_string());
        assert_eq!(to_string(&Bool(true)).unwrap(), "true".to_string());
        assert_eq!(to_string(&Bool(false)).unwrap(), "false".to_string());
        assert_eq!(to_string(&Int(42)).unwrap(), "42".to_string());
        assert_eq!(to_string(&Double(3.1415)).unwrap(), "3.1415".to_string());
        assert_eq!(
            to_string(&String("Hello, World!".into())).unwrap(),
            "\"Hello, World!\""
        );
        assert_eq!(
            to_string(&Data(vec![192, 168, 0, 1])).unwrap(),
            ".Data(\"wKgAAQ==\")"
        );
    }

    #[test]
    fn test_serialize_compound() {
        use super::Value::*;
        use to_string;

        assert_eq!(to_string(&Map(::sequence::Map { contents: vec![
            (String("array".into()), Array(::sequence::Array {
                contents: vec![
                    Nil,
                    Bool(true),
                    Int(1),
                    Double(1.1),
                    String("one".into()),
                    Array(::sequence::Array { contents: vec![Int(1)] }),
                    Map(::sequence::Map { contents: vec![(String("one".into()), Double(1.1))] }),
                ]
            })),
            (Nil, String("Unlike JSON and Property Lists,".into())),
            (Bool(true), String("Yes, SION".into())),
            (Int(1), String("does accept".into())),
            (Double(1.1), String("non-String keys.".into())),
            (Array(::sequence::Array { contents: vec![] }), String("like".into())),
            (Map(::sequence::Map { contents: vec![] }), String("Map of ECMAScript.".into())),
        ]})).unwrap(), r#"["array":[nil,true,1,1.1,"one",[1],["one":1.1]],nil:"Unlike JSON and Property Lists,",true:"Yes, SION",1:"does accept",1.1:"non-String keys.",[]:"like",[:]:"Map of ECMAScript."]"#);
    }

    #[test]
    fn test_deserialize_simple() {
        use super::Value::{self, *};
        use from_str;

        assert_eq!(from_str::<Value>("nil").unwrap(), Nil);
        assert_eq!(from_str::<Value>("true").unwrap(), Bool(true));
        assert_eq!(from_str::<Value>("false").unwrap(), Bool(false));
        assert_eq!(from_str::<Value>("42").unwrap(), Int(42));
        assert_eq!(from_str::<Value>("3.1415").unwrap(), Double(3.1415));
        assert_eq!(
            from_str::<Value>("\"Hello, World!\"").unwrap(),
            String("Hello, World!".into())
        );
        assert_eq!(
            from_str::<Value>(".Data(\"wKgAAQ==\")").unwrap(),
            Data(vec![192, 168, 0, 1])
        );
    }
}
