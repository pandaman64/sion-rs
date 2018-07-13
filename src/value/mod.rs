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

    #[test]
    fn test_deserialize_compound() {
        use super::Value::{self, *};
        use from_str;
        use sequence::{Array as A, Map as M};

        let input = r#"
[
    "array": [
        nil,
        true,
        1,      // Int in decimal
        1.0,    // Double in decimal
        "one",
        [1],
        ["one" : 1.0]
    ],
    "bool": true,
    "data": .Data("R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7"),
    "dictionary": [
        "array" : [],
        "bool" : false,
        "double" : 0.0,
        "int" : 0,
        "nil" : nil,
        "object" : [:],
        "string" : ""
    ],
    "string": "漢字、カタカナ、ひらがなの入ったstring😇",
    "url": "https://github.com/dankogai/",
    nil   : "Unlike JSON and Property Lists,",
    true  : "Yes, SION",
    1     : "does accept",
    1.0   : "non-String keys.",
    []    : "like",
    [:]   : "Map of ECMAScript."
]"#;
        let expected = Map(M {
            contents: vec![
                (
                    String("array".into()),
                    Array(A {
                        contents: vec![
                            Nil,
                            Bool(true),
                            Int(1),
                            Double(1.0),
                            String("one".into()),
                            Array(A {
                                contents: vec![Int(1)],
                            }),
                            Map(M {
                                contents: vec![(String("one".into()), Double(1.0))],
                            }),
                        ],
                    }),
                ),
                (String("bool".into()), Bool(true)),
                (
                    String("data".into()),
                    Data(vec![
                        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00,
                        0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
                        0x00, 0x02, 0x01, 0x44, 0x00, 0x3b,
                    ]),
                ),
                (
                    String("dictionary".into()),
                    Map(M {
                        contents: vec![
                            (String("array".into()), Array(A { contents: vec![] })),
                            (String("bool".into()), Bool(false)),
                            (String("double".into()), Double(0.0)),
                            (String("int".into()), Int(0)),
                            (String("nil".into()), Nil),
                            (String("object".into()), Map(M { contents: vec![] })),
                            (String("string".into()), String("".into())),
                        ],
                    }),
                ),
                (
                    String("string".into()),
                    String("漢字、カタカナ、ひらがなの入ったstring😇".into()),
                ),
                (
                    String("url".into()),
                    String("https://github.com/dankogai/".into()),
                ),
                (Nil, String("Unlike JSON and Property Lists,".into())),
                (Bool(true), String("Yes, SION".into())),
                (Int(1), String("does accept".into())),
                (Double(1.0), String("non-String keys.".into())),
                (
                    Array(::sequence::Array { contents: vec![] }),
                    String("like".into()),
                ),
                (
                    Map(::sequence::Map { contents: vec![] }),
                    String("Map of ECMAScript.".into()),
                ),
            ],
        });
        assert_eq!(from_str::<Value>(input).unwrap(), expected);
    }
}
