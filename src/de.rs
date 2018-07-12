use super::{Error, Result};
use serde::de::{Deserialize, Visitor};

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

    // skip until one of "\r", "\n", "\r\n"
    fn skip_line(&mut self) -> Result<()> {
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
        Ok(())
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
            // comment
            '/' => {
                self.expect('/', self::Error::ExpectSlash)?;
                self.skip_line()?;
                return self.deserialize_any(visitor);
            }
            // string
            '"' => unimplemented!(),
            // an array or a map
            '[' => unimplemented!(),
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
                    Int(i) => visitor.visit_i64(i),
                    Double(f) => visitor.visit_f64(f),
                }
            },
            // double, data, date
            '.' => unimplemented!(),
            c if c.is_whitespace() => {
                self.skip()?;
                return self.deserialize_any(visitor);
            }
            _ => unimplemented!(),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
