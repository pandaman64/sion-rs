use super::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ParseResult {
    Int(i64),
    Double(f64),
}

// TODO: overflow check
fn parse_int(radix: Radix, s: &str) -> i64 {
    use self::Radix::Hexadecimal;

    let mut val = 0;
    for c in s.chars() {
        val *= i64::from(radix.radix());
        match c {
            '0' | '_' => {}
            '1' => val += 1,
            '2' => val += 2,
            '3' => val += 3,
            '4' => val += 4,
            '5' => val += 5,
            '6' => val += 6,
            '7' => val += 7,
            '8' => val += 8,
            '9' => val += 9,
            'a' | 'A' if radix == Hexadecimal => val += 10,
            'b' | 'B' if radix == Hexadecimal => val += 11,
            'c' | 'C' if radix == Hexadecimal => val += 12,
            'd' | 'D' if radix == Hexadecimal => val += 13,
            'e' | 'E' if radix == Hexadecimal => val += 14,
            'f' | 'F' if radix == Hexadecimal => val += 15,
            // TODO: return Err
            _ => unreachable!("{:?}, {}", radix, c),
        }
    }
    val
}

// FIXME: toy implementation
impl ParseResult {
    fn integer(sign: Sign, radix: Radix, s: &str) -> Self {
        use self::ParseResult::Int;
        if sign == self::Sign::Positive {
            Int(parse_int(radix, s))
        } else {
            Int(-parse_int(radix, s))
        }
    }

    fn double(
        sign: Sign,
        radix: Radix,
        integer: Option<&str>,
        fractional: &str,
        exponent_sign: Sign,
        exponent: &str,
    ) -> Self {
        use self::ParseResult::Double;

        let integer = integer
            .map(|integer| parse_int(radix, integer) as f64)
            .unwrap_or(0.0);
        let fractional = parse_int(radix, fractional) as f64;
        let exponent = parse_int(radix, exponent) as i32;

        let base = if fractional != 0.0 {
            integer
                + fractional
                    / f64::from(radix.radix()).powf(fractional.log(f64::from(radix.radix())).ceil())
        } else {
            integer
        };
        let v = if exponent_sign == self::Sign::Positive {
            base * f64::from(radix.exponent()).powi(exponent)
        } else {
            base * f64::from(radix.exponent()).powi(-exponent)
        };
        if sign == self::Sign::Positive {
            Double(v)
        } else {
            Double(-v)
        }
    }
}

type Literal = (usize, usize);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Sign {
    Positive,
    Negative,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Radix {
    Decimal,
    Hexadecimal,
}
impl Radix {
    fn radix(&self) -> u32 {
        use self::Radix::*;
        match self {
            Decimal => 10,
            Hexadecimal => 16,
        }
    }

    fn is_exponent(&self, c: char) -> bool {
        use self::Radix::*;
        match self {
            Decimal => c == 'e' || c == 'E',
            Hexadecimal => c == 'p' || c == 'P',
        }
    }

    fn exponent(&self) -> u32 {
        use self::Radix::*;
        match self {
            Decimal => 10,
            Hexadecimal => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParserState {
    Start,
    NegStart,
    LeadingZero {
        sign: Sign,
        position: usize,
    },
    HexadecimalStart {
        sign: Sign,
    },
    IntOrDouble {
        sign: Sign,
        radix: Radix,
        integer: Literal,
    },
    Double {
        sign: Sign,
        radix: Radix,
        integer: Option<Literal>,
        fraction: Option<Literal>,
    },
    DoubleExponent {
        sign: Sign,
        radix: Radix,
        integer: Option<Literal>,
        fraction: Literal,
        exponent_sign: Option<Sign>,
        exponent: Option<Literal>,
    },
    Done,
}

impl ParserState {
    fn get_result(self, s: &str) -> Result<ParseResult> {
        use self::ParserState::*;

        match self {
            Start | NegStart | HexadecimalStart { .. } | Done => unreachable!(),
            LeadingZero { .. } => Ok(ParseResult::Int(0)),
            IntOrDouble {
                sign,
                radix,
                integer: (start, end),
            } => Ok(ParseResult::integer(sign, radix, &s[start..end])),
            Double {
                sign,
                radix,
                integer,
                fraction: Some(fraction),
            } => Ok(ParseResult::double(
                sign,
                radix,
                integer.map(|integer| &s[integer.0..integer.1]),
                &s[fraction.0..fraction.1],
                self::Sign::Positive,
                "0",
            )),
            Double { fraction: None, .. } => Err(self::Error::ExpectedFraction),
            DoubleExponent {
                sign,
                radix,
                integer,
                fraction,
                exponent_sign,
                exponent: Some(exponent),
            } => Ok(ParseResult::double(
                sign,
                radix,
                integer.map(|integer| &s[integer.0..integer.1]),
                &s[fraction.0..fraction.1],
                exponent_sign.unwrap_or(self::Sign::Positive),
                &s[exponent.0..exponent.1],
            )),
            DoubleExponent { exponent: None, .. } => Err(self::Error::ExpectedExponent),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Parser<'de> {
    original: &'de str,
    input: &'de str,
    state: ParserState,
}

impl<'de> Parser<'de> {
    pub(crate) fn new(input: &'de str) -> Self {
        Parser {
            original: input,
            input,
            state: self::ParserState::Start,
        }
    }

    pub(crate) fn get_output(&self) -> &'de str {
        self.input
    }

    fn peek(&self) -> Result<char> {
        self.input.chars().next().ok_or(self::Error::Eof)
    }

    fn skip(&mut self) -> Result<()> {
        let c = self.peek()?;
        self.input = &self.input[c.len_utf8()..];
        Ok(())
    }

    fn expect(&mut self, expected: &str, e: Error) -> Result<()> {
        if self.input.starts_with(expected) {
            self.input = &self.input[expected.len()..];
            Ok(())
        } else {
            Err(e)
        }
    }

    pub(crate) fn run(&mut self) -> Result<ParseResult> {
        use self::ParserState::*;
        use self::Radix::*;

        let mut char_indices = self.input.char_indices();

        while let Some((i, c)) = char_indices.next() {
            match ::std::mem::replace(&mut self.state, Done) {
                Start => {
                    if c == '0' {
                        self.state = LeadingZero {
                            sign: self::Sign::Positive,
                            position: i,
                        };
                        self.skip()?;
                    } else if c.is_digit(10) {
                        self.state = IntOrDouble {
                            sign: self::Sign::Positive,
                            radix: Decimal,
                            integer: (i, i + c.len_utf8()),
                        };
                        self.skip()?;
                    } else if c == 'N' {
                        self.expect("NaN", self::Error::ExpectedNaN)?;
                        return Ok(ParseResult::Double(::std::f64::NAN));
                    } else if c == 'I' {
                        self.expect("Infinity", self::Error::ExpectedInfinity)?;
                        return Ok(ParseResult::Double(::std::f64::INFINITY));
                    } else if c == '-' {
                        self.state = NegStart;
                        self.skip()?;
                    } else if c == '.' {
                        self.state = Double {
                            sign: self::Sign::Positive,
                            radix: Decimal,
                            integer: None,
                            fraction: None,
                        };
                        self.skip()?;
                    } else {
                        return Err(self::Error::Expected(
                            "an integer, NaN, Infinity, -, .".into(),
                        ));
                    }
                }
                LeadingZero { sign, position } => {
                    if c == 'x' {
                        self.state = HexadecimalStart { sign };
                        self.skip()?;
                    } else if c == '.' {
                        self.state = Double {
                            sign,
                            radix: Decimal,
                            integer: Some((position, position + '0'.len_utf8())),
                            fraction: None,
                        };
                        self.skip()?;
                    } else if c.is_digit(10) {
                        self.state = IntOrDouble {
                            sign,
                            radix: Decimal,
                            integer: (position, position + '0'.len_utf8() + c.len_utf8()),
                        };
                        self.skip()?;
                    } else {
                        return LeadingZero { sign, position }.get_result(self.original);
                    }
                }
                NegStart => {
                    if c == '0' {
                        self.state = LeadingZero {
                            sign: self::Sign::Negative,
                            position: i,
                        };
                    } else if c.is_digit(10) {
                        self.state = IntOrDouble {
                            sign: self::Sign::Negative,
                            radix: Decimal,
                            integer: (i, i + c.len_utf8()),
                        };
                        self.skip()?;
                    } else if c == 'I' {
                        self.expect("Infinity", self::Error::ExpectedInfinity)?;
                        return Ok(ParseResult::Double(::std::f64::NEG_INFINITY));
                    } else if c == '.' {
                        self.state = Double {
                            sign: self::Sign::Negative,
                            radix: Decimal,
                            integer: None,
                            fraction: None,
                        };
                        self.skip()?;
                    } else {
                        return Err(self::Error::Expected("an integer, Infinity, .".into()));
                    }
                }
                HexadecimalStart { sign } => {
                    if c.is_digit(16) {
                        self.state = IntOrDouble {
                            sign,
                            radix: Hexadecimal,
                            integer: (i, i + c.len_utf8()),
                        };
                        self.skip()?;
                    } else {
                        return Err(self::Error::ExpectedHexadecimalDigit);
                    }
                }
                IntOrDouble {
                    sign,
                    radix,
                    integer: (start, end),
                } => {
                    if c.is_digit(radix.radix()) || c == '_' {
                        self.state = IntOrDouble {
                            sign,
                            radix,
                            integer: (start, end + c.len_utf8()),
                        };
                        self.skip()?;
                    } else if c == '.' {
                        self.state = Double {
                            sign,
                            radix,
                            integer: Some((start, end)),
                            fraction: None,
                        };
                        self.skip()?;
                    } else {
                        return IntOrDouble {
                            sign,
                            radix,
                            integer: (start, end),
                        }.get_result(self.original);
                    }
                }
                Double {
                    sign,
                    radix,
                    integer,
                    fraction,
                } => {
                    if c.is_digit(radix.radix()) {
                        if let Some((start, end)) = fraction {
                            self.state = Double {
                                sign,
                                radix,
                                integer,
                                fraction: Some((start, end + c.len_utf8())),
                            };
                            self.skip()?;
                        } else {
                            self.state = Double {
                                sign,
                                radix,
                                integer,
                                fraction: Some((i, i + c.len_utf8())),
                            };
                            self.skip()?;
                        }
                    } else if c == '_' {
                        if let Some((start, end)) = fraction {
                            self.state = Double {
                                sign,
                                radix,
                                integer,
                                fraction: Some((start, end + c.len_utf8())),
                            };
                            self.skip()?;
                        } else {
                            return Err(self::Error::UnexpectedUnderBar);
                        }
                    } else if radix.is_exponent(c) {
                        if let Some(fraction) = fraction {
                            self.state = DoubleExponent {
                                sign,
                                radix,
                                integer,
                                fraction,
                                exponent_sign: None,
                                exponent: None,
                            };
                            self.skip()?;
                        } else {
                            return Err(self::Error::ExpectedFraction);
                        }
                    } else {
                        return Double {
                            sign,
                            radix,
                            integer,
                            fraction,
                        }.get_result(self.original);
                    }
                }
                DoubleExponent {
                    sign,
                    radix,
                    integer,
                    fraction,
                    exponent_sign,
                    exponent,
                } => {
                    if c == '+' || c == '-' {
                        if exponent_sign.is_none() && exponent.is_none() {
                            self.state = DoubleExponent {
                                sign,
                                radix,
                                integer,
                                fraction,
                                exponent_sign: Some(if c == '+' {
                                    self::Sign::Positive
                                } else {
                                    self::Sign::Negative
                                }),
                                exponent: None,
                            };
                            self.skip()?;
                        } else {
                            return Err(self::Error::UnexpectedSign);
                        }
                    } else if c.is_digit(10) {
                        if let Some((start, end)) = exponent {
                            self.state = DoubleExponent {
                                sign,
                                radix,
                                integer,
                                fraction,
                                exponent_sign,
                                exponent: Some((start, end + c.len_utf8())),
                            };
                            self.skip()?;
                        } else {
                            self.state = DoubleExponent {
                                sign,
                                radix,
                                integer,
                                fraction,
                                exponent_sign,
                                exponent: Some((i, i + c.len_utf8())),
                            };
                            self.skip()?;
                        }
                    } else if c == '_' {
                        if let Some((start, end)) = exponent {
                            self.state = DoubleExponent {
                                sign,
                                radix,
                                integer,
                                fraction,
                                exponent_sign,
                                exponent: Some((start, end + c.len_utf8())),
                            };
                            self.skip()?;
                        } else {
                            return Err(self::Error::UnexpectedUnderBar);
                        }
                    } else {
                        return DoubleExponent {
                            sign,
                            radix,
                            integer,
                            fraction,
                            exponent_sign,
                            exponent,
                        }.get_result(self.original);
                    }
                }
                // TODO: handle better
                Done => unreachable!(),
            }
        }

        ::std::mem::replace(&mut self.state, Done).get_result(self.original)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_integer() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("1234567890remaining");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "remaining");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Int(1234567890));
    }

    #[test]
    fn test_double() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("1234.567890remaining");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "remaining");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Double(1234.567890));
    }

    #[test]
    fn test_hex() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("0xffFF");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Int(65535));
    }

    #[test]
    fn test_exponent() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("1.25e-4");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Double(1.25e-4));
    }

    #[test]
    fn test_hex_exponent() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("0x1.2p-2");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Double(0.28125));
    }

    #[test]
    fn test_nan() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("NaNhoge");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "hoge");
        assert_eq!(parser.state, Done);

        match result {
            Double(v) => assert!(v.is_nan()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_infinity() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("Infinityhoge");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "hoge");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Double(::std::f64::INFINITY));
    }

    #[test]
    fn test_neg_infinity() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("-Infinityhoge");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "hoge");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Double(::std::f64::NEG_INFINITY));
    }

    #[test]
    fn test_negative_dot() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("-.1234hoge");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "hoge");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Double(-0.1234));
    }

    #[test]
    fn test_one_point_zero() {
        use super::ParseResult::*;
        use super::Parser;
        use super::ParserState::Done;

        let mut parser = Parser::new("1.0");
        let result = parser.run().unwrap();
        assert_eq!(parser.input, "");
        assert_eq!(parser.state, Done);
        assert_eq!(result, Double(1.0));
    }
}
