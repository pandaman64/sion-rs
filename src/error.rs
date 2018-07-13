use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    SerializeError(String),
    DeserializeError(String),
    TrailingCharacters,
    Eof,
    ExpectSlash,
    ExpectInfinity,
    ExpectNaN,
    ExpectFraction,
    ExpectExponent,
    ExpectDecimalDigit,
    ExpectHexadecimalDigit,
    ExpectOpenBracket,
    ExpectCloseBracket,
    ExpectComma,
    ExpectColon,
    ExpectDouble(i64),
    ExpectNil,
    ExpectTrue,
    ExpectFalse,
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
            ExpectSlash => write!(f, "expected a slash"),
            ExpectInfinity => write!(f, "expected Infinity"),
            ExpectNaN => write!(f, "expected NaN"),
            ExpectFraction => write!(f, "expected fractional part"),
            ExpectExponent => write!(f, "expected exponent part"),
            ExpectDecimalDigit => write!(f, "expected decimal digit"),
            ExpectHexadecimalDigit => write!(f, "expected hexadecimal digit"),
            ExpectOpenBracket => write!(f, "expected open bracket"),
            ExpectCloseBracket => write!(f, "expected close bracket"),
            ExpectComma => write!(f, "expected comma"),
            ExpectColon => write!(f, "expected colon"),
            ExpectDouble(x) => write!(f, "expected double: {}", x),
            ExpectNil => write!(f, "expected nil"),
            ExpectTrue => write!(f, "expected true"),
            ExpectFalse => write!(f, "expected false"),
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
