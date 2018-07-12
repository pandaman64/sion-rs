use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    SerializeError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            SerializeError(s) => write!(f, "serialization error: {}", s),
        }
    }
}

impl ::std::error::Error for Error {}

impl ::serde::ser::Error for Error {
    fn custom<T: Display>(v: T) -> Error {
        Error::SerializeError(v.to_string())
    }
}
