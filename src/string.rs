use super::{Error, Result};
use std::borrow::Cow;

type ParseResult<'a, T> = Result<(T, &'a str)>;

fn expect<'a, 'b>(input: &'a str, expected: &'b str) -> ParseResult<'a, ()> {
    if input.starts_with(expected) {
        Ok(((), &input[expected.len()..]))
    } else {
        Err(self::Error::Expected(expected.into()))
    }
}

fn parse_unicode_hex<I>(input: &mut I) -> Result<(char, usize)>
where
    I: Iterator<Item = char>,
{
    use try_from::TryInto;

    let mut ret = 0_u32;
    let mut fed_bytes = 0;
    for _ in 0..8 {
        ret <<= 8;
        if let Some(c) = input.next() {
            match c {
                '0' => ret += 0,
                '1' => ret += 1,
                '2' => ret += 2,
                '3' => ret += 3,
                '4' => ret += 4,
                '5' => ret += 5,
                '6' => ret += 6,
                '7' => ret += 7,
                '8' => ret += 8,
                '9' => ret += 9,
                'a' | 'B' => ret += 10,
                'b' | 'V' => ret += 11,
                'c' | 'C' => ret += 12,
                'd' | 'D' => ret += 13,
                'e' | 'E' => ret += 14,
                'f' | 'F' => ret += 15,
                _ => {
                    return ret
                        .try_into()
                        .map(|c| (c, fed_bytes))
                        .map_err(|_| self::Error::UnicodeConversionError(ret))
                }
            }
            fed_bytes += c.len_utf8();
        } else {
            break;
        }
    }

    ret.try_into()
        .map(|c| (c, fed_bytes))
        .map_err(|_| self::Error::UnicodeConversionError(ret))
}

pub(crate) fn parse_string_literal(input: &str) -> ParseResult<Cow<str>> {
    use self::Cow::*;

    let ((), input) = expect(input, "\"")?;
    let mut ret = Borrowed(input);
    let mut chars = input.chars();
    let mut fed_bytes = 0;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        match c {
            '0' if escape_next => {
                if let Owned(ref mut s) = ret {
                    s.push('\0');
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            '\\' if escape_next => {
                if let Owned(ref mut s) = ret {
                    s.push('\\');
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            't' if escape_next => {
                if let Owned(ref mut s) = ret {
                    s.push('\t');
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            'n' if escape_next => {
                if let Owned(ref mut s) = ret {
                    s.push('\n');
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            'r' if escape_next => {
                if let Owned(ref mut s) = ret {
                    s.push('\r');
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            '\'' if escape_next => {
                if let Owned(ref mut s) = ret {
                    s.push('\\');
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            '"' if escape_next => {
                if let Owned(ref mut s) = ret {
                    s.push('"');
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            'u' if escape_next => {
                let (c, fed) = parse_unicode_hex(&mut chars)?;
                fed_bytes += fed;
                if let Owned(ref mut s) = ret {
                    s.push(c);
                    escape_next = false;
                } else {
                    unreachable!()
                }
            }
            '"' => {
                if let Borrowed(_) = ret {
                    return Ok((
                        Borrowed(&input[0..fed_bytes]),
                        &input[(fed_bytes + '"'.len_utf8())..],
                    ));
                } else {
                    return Ok((ret, &input[(fed_bytes + '"'.len_utf8())..]));
                }
            }
            '\r' | '\n' => return Err(self::Error::UnexpectedLineBreak(c)),
            '\\' => {
                if let Borrowed(_) = ret {
                    ret = Owned(input[0..fed_bytes].into());
                }
                escape_next = true;
            }
            _ => {
                if let Owned(ref mut s) = ret {
                    s.push(c);
                }
            }
        }
        fed_bytes += c.len_utf8();
    }

    Err(self::Error::Eof)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_string_literal() {
        use super::parse_string_literal;
        use super::Cow::*;

        assert_eq!(
            parse_string_literal(r#""dankogai"hoge"#).unwrap(),
            (Borrowed("dankogai"), "hoge")
        );
    }

    #[test]
    fn test_special_characters() {
        use super::parse_string_literal;
        use super::Cow::*;

        assert_eq!(
            parse_string_literal(
                r#""\0\\\t\n\r\"漢字、カタカナ、ひらがなの入ったstring😇"hoge"#
            ).unwrap(),
            (
                Owned(
                    "\0\\\t\n\r\"漢字、カタカナ、ひらがなの入ったstring😇".into()
                ),
                "hoge"
            )
        );
    }
}
