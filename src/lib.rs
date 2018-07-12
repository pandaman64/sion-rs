#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate failure;
extern crate try_from;

pub mod de;
pub mod error;
mod number;
pub mod sequence;
pub mod ser;
mod string;
pub mod value;

pub use de::from_str;
pub use error::Error;
pub use ser::to_string;

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use from_str;
    use to_string;

    #[test]
    fn test_basic() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Person {
            name: String,
            age: u32,
        }

        let person = Person {
            name: "dankogai".into(),
            age: 48,
        };
        let expected = r#"["name":"dankogai","age":48]"#;
        assert_eq!(to_string(&person).unwrap(), expected);

        let deserialized = from_str::<Person>(expected).unwrap();
        assert_eq!(deserialized, person);
    }

    #[test]
    fn test_compound() {
        use std::collections::HashMap;
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Language {
            name: String,
            tag: Vec<String>,
        }

        let mut compilers = HashMap::new();
        compilers.insert(
            "rustc".into(),
            Language {
                name: "Rust".into(),
                tag: vec!["system programming".into(), "Mozilla".into()],
            },
        );
        compilers.insert(
            "clang".into(),
            Language {
                name: "C++".into(),
                tag: vec![],
            },
        );

        let serialized = to_string(&compilers).unwrap();
        let deserialized: HashMap<String, Language> = from_str(&serialized).unwrap();
        assert_eq!(deserialized, compilers);
    }
}
