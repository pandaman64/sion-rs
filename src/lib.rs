extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate failure;

pub mod error;
pub mod ser;
pub mod value;

pub use error::Error;
pub use ser::to_string;

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use to_string;

    #[test]
    fn test_basic() {
        #[derive(Serialize)]
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
    }
}
