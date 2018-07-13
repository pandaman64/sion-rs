use super::{Error, Result};
use serde::de::{Deserialize, DeserializeSeed, MapAccess, SeqAccess, Visitor};

#[derive(Debug)]
pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

pub fn from_str<'a, T>(input: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(input);
    let v = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(v)
    } else {
        Err(self::Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn peek(&self) -> Result<char> {
        self.input.chars().next().ok_or(self::Error::Eof)
    }

    fn skip(&mut self) -> Result<()> {
        let c = self.peek()?;
        self.input = &self.input[c.len_utf8()..];
        Ok(())
    }

    fn expect(&mut self, expected: char, e: Error) -> Result<()> {
        let got = self.peek()?;
        if got == expected {
            self.skip()?;
            Ok(())
        } else {
            Err(e)
        }
    }

    // skip until feed one of "\r", "\n", "\r\n"
    fn skip_line(&mut self) {
        enum State {
            Normal(usize),
            EncounterCR(usize),
        }

        impl State {
            fn fed_bytes(&self) -> usize {
                match self {
                    State::Normal(l) | State::EncounterCR(l) => *l,
                }
            }

            fn feed(&mut self, c: char) {
                match self {
                    State::Normal(ref mut l) | State::EncounterCR(ref mut l) => *l += c.len_utf8(),
                }
            }
        }

        let mut state = State::Normal(0);
        let mut chars = self.input.chars();

        while let Some(c) = chars.next() {
            if c == '\n' {
                state.feed('\n');
                break;
            }

            match state {
                State::Normal(_) => state.feed(c),
                State::EncounterCR(_) => break,
            }

            if c == '\r' {
                state = State::EncounterCR(state.fed_bytes());
            }
        }

        self.input = &self.input[state.fed_bytes()..];
    }

    fn trim(&mut self) -> Result<()> {
        match self.peek() {
            // comment
            Ok('/') => {
                self.skip()?;
                self.expect('/', self::Error::ExpectedSlash)?;
                self.skip_line();
                self.trim()
            }
            // whitespaces
            Ok(c) if c.is_whitespace() => {
                self.skip()?;
                self.trim()
            }
            _ => Ok(()),
        }
    }
}

impl<'de, 'a> ::serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let first = self.peek()?;
        match first {
            // comment or white spaces
            c if c == '/' || c.is_whitespace() => {
                self.trim()?;
                self.deserialize_any(visitor)
            }
            // nil
            'n' => {
                if self.input.starts_with("nil") {
                    self.input = &self.input["nil".len()..];
                    visitor.visit_unit()
                } else {
                    Err(self::Error::ExpectedNil)
                }
            }
            // true
            't' => {
                if self.input.starts_with("true") {
                    self.input = &self.input["true".len()..];
                    visitor.visit_bool(true)
                } else {
                    Err(self::Error::ExpectedTrue)
                }
            }
            // false
            'f' => {
                if self.input.starts_with("false") {
                    self.input = &self.input["false".len()..];
                    visitor.visit_bool(false)
                } else {
                    Err(self::Error::ExpectedFalse)
                }
            }
            // string
            '"' => {
                use ::std::borrow::Cow::*;
                match ::string::parse_string_literal(self.input)? {
                    (Borrowed(s), output) => {
                        self.input = output;
                        visitor.visit_borrowed_str(s)
                    },
                    (Owned(s), output) => {
                        self.input = output;
                        visitor.visit_string(s)
                    }
                }
            },
            // array or map
            '[' => {
                // FIXME: VERY INEFFICIENT: deserialize a Value and throw it away!
                // create a deserializer to look ahead
                let mut deserializer = Deserializer { input: self.input };
                deserializer.skip()?;
                deserializer.trim()?;

                // empty sequence
                match deserializer.peek()? {
                    ']' => return self.deserialize_seq(visitor),
                    ':' => return self.deserialize_map(visitor),
                    _ => {},
                }

                // this sequence has a value, parse it and discard
                ::value::Value::deserialize(&mut deserializer)?;
                deserializer.trim()?;
                match deserializer.peek()? {
                    ',' | ']' => self.deserialize_seq(visitor),
                    ':' => self.deserialize_map(visitor),
                    _ => Err(self::Error::Expected("',', ], :".into()))
                }
            },
            // int or double
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
            // NaN
            | 'N'
            // Infinity
            | 'I'
            // int, double, -Infinity,
            | '-' => {
                use number::{Parser, ParseResult::*};

                let mut parser = Parser::new(self.input);
                match parser.run()? {
                    Int(i) => {
                        self.input = parser.get_output();
                        visitor.visit_i64(i)
                    },
                    Double(f) => {
                        self.input = parser.get_output();
                        visitor.visit_f64(f)
                    },
                }
            },
            // double, data, date
            '.' => {
                use number::{Parser, ParseResult::*};

                let mut parser = Parser::new(self.input);
                match parser.run() {
                    Ok(Int(_)) => unreachable!(),
                    Ok(Double(f)) => {
                        self.input = parser.get_output();
                        visitor.visit_f64(f)
                    },
                    Err(_) => {
                        self.skip()?;
                        if self.input.starts_with("Data") {
                            self.input = &self.input["Data".len()..];
                            self.trim()?;
                            self.expect('(', self::Error::ExpectedOpenBracket)?;
                            self.trim()?;
                            let (literal, output) = ::string::parse_string_literal(self.input)?;
                            self.input = output;
                            let data = ::base64::decode(literal.as_bytes())
                                .map_err(|_| self::Error::Base64DecodeError)?;
                            self.trim()?;
                            self.expect(')', self::Error::ExpectedCloseBracket)?;

                            visitor.visit_bytes(&data)
                        } else if self.input.starts_with("Date") {
                            self.input = &self.input["Date".len()..];
                            self.trim()?;
                            self.expect('(', self::Error::ExpectedOpenBracket)?;
                            self.trim()?;
                            let mut parser = Parser::new(self.input);
                            match parser.run()? {
                                Int(x) => Err(self::Error::ExpectedDouble(x)),
                                Double(f) => {
                                    self.trim()?;
                                    self.expect(')', self::Error::ExpectedCloseBracket)?;

                                    visitor.visit_f64(f)
                                }
                            }
                        } else {
                            Err(self::Error::Expected("Double, Data, or Date".into()))
                        }
                    }
                }
            },
            c => unreachable!("{}", c),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect('[', self::Error::ExpectedOpenBracket)?;
        let seq = visitor.visit_seq(CommaSeparated::new(&mut *self))?;
        self.expect(']', self::Error::ExpectedCloseBracket)?;
        Ok(seq)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.expect('[', self::Error::ExpectedOpenBracket)?;
        let map = visitor.visit_map(CommaSeparated::new(&mut *self))?;
        self.expect(']', self::Error::ExpectedCloseBracket)?;
        Ok(map)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct
        enum identifier ignored_any
    }
}

#[derive(Debug)]
struct CommaSeparated<'a, 'de: 'a> {
    deserializer: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de: 'a> CommaSeparated<'a, 'de> {
    fn new(deserializer: &'a mut Deserializer<'de>) -> Self {
        CommaSeparated {
            deserializer,
            first: true,
        }
    }
}

impl<'a, 'de: 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        self.deserializer.trim()?;
        if self.deserializer.peek()? == ']' {
            return Ok(None);
        }
        if !self.first {
            self.deserializer.trim()?;
            self.deserializer.expect(',', self::Error::ExpectedComma)?;
        }
        self.first = false;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }
}

impl<'a, 'de: 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        self.deserializer.trim()?;
        match self.deserializer.peek()? {
            ']' if !self.first => return Ok(None),
            ':' if self.first => {
                self.deserializer.skip()?;
                return Ok(None);
            }
            _ => {}
        }
        if !self.first {
            self.deserializer.trim()?;
            self.deserializer.expect(',', self::Error::ExpectedComma)?;
        }
        self.first = false;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.deserializer.trim()?;
        self.deserializer.expect(':', self::Error::ExpectedColon)?;
        seed.deserialize(&mut *self.deserializer)
    }
}
