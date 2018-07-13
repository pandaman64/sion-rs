use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    SerializeError(String),
    DeserializeError(String),
    TrailingCharacters,
    Eof,
    ExpectedSlash,
    ExpectedInfinity,
    ExpectedNaN,
    ExpectedFraction,
    ExpectedExponent,
    ExpectedDecimalDigit,
    ExpectedHexadecimalDigit,
    ExpectedOpenBracket,
    ExpectedCloseBracket,
    ExpectedComma,
    ExpectedColon,
    ExpectedDouble(i64),
    ExpectedNil,
    ExpectedTrue,
    ExpectedFalse,
    Expected(String),
    UnexpectedUnderBar,
    UnexpectedSign,
    UnexpectedLineBreak(char),
    UnexpectedOpenBracket,
    UnicodeConversionError(u32),
    Base64DecodeError,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            SerializeError(s) => write!(f, "serialization error: {}", s),
            DeserializeError(s) => write!(f, "deserialization error: {}", s),
            TrailingCharacters => write!(f, "input has trailing characters"),
            Eof => write!(f, "encountered EOF while parsing"),
            ExpectedSlash => write!(f, "expected a slash"),
            ExpectedInfinity => write!(f, "expected Infinity"),
            ExpectedNaN => write!(f, "expected NaN"),
            ExpectedFraction => write!(f, "expected fractional part"),
            ExpectedExponent => write!(f, "expected exponent part"),
            ExpectedDecimalDigit => write!(f, "expected decimal digit"),
            ExpectedHexadecimalDigit => write!(f, "expected hexadecimal digit"),
            ExpectedOpenBracket => write!(f, "expected open bracket"),
            ExpectedCloseBracket => write!(f, "expected close bracket"),
            ExpectedComma => write!(f, "expected comma"),
            ExpectedColon => write!(f, "expected colon"),
            ExpectedDouble(x) => write!(f, "expected double: {}", x),
            ExpectedNil => write!(f, "expected nil"),
            ExpectedTrue => write!(f, "expected true"),
            ExpectedFalse => write!(f, "expected false"),
            Expected(s) => write!(f, "expected: {}", s),
            UnexpectedUnderBar => write!(f, "unexpected under bar"),
            UnexpectedSign => write!(f, "unexpected sign"),
            UnexpectedLineBreak(c) => write!(f, "unexpected line break: {}", c.escape_debug()),
            UnexpectedOpenBracket => write!(f, "unexpected open bracket"),
            UnicodeConversionError(x) => write!(f, "failed to convert into unicode: {}", x),
            Base64DecodeError => write!(f, "failed to decode base64"),
        }
    }
}

impl ::std::error::Error for Error {}

impl ::serde::ser::Error for Error {
    fn custom<T: Display>(v: T) -> Error {
        Error::SerializeError(v.to_string())
    }
}

impl ::serde::de::Error for Error {
    fn custom<T: Display>(v: T) -> Error {
        Error::DeserializeError(v.to_string())
    }
}
