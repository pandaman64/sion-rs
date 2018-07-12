pub mod ser;

#[derive(Debug)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Data(Vec<u8>),
    Date(f64),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
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

        assert_eq!(to_string(&Map(vec![
            (String("array".into()), Array(vec![
                Nil,
                Bool(true),
                Int(1),
                Double(1.1),
                String("one".into()),
                Array(vec![Int(1)]),
                Map(vec![(String("one".into()), Double(1.1))]),
            ])),
            (Nil, String("Unlike JSON and Property Lists,".into())),
            (Bool(true), String("Yes, SION".into())),
            (Int(1), String("does accept".into())),
            (Double(1.1), String("non-String keys.".into())),
            (Array(vec![]), String("like".into())),
            (Map(vec![]), String("Map of ECMAScript.".into())),
        ])).unwrap(), r#"["array":[nil,true,1,1.1,"one",[1],["one":1.1]],nil:"Unlike JSON and Property Lists,",true:"Yes, SION",1:"does accept",1.1:"non-String keys.",[]:"like",[:],"Map of ECMAScript."]"#);
    }
}
