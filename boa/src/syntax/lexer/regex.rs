//! This module implements lexing for regex literals used in the JavaScript programing language.

use super::{Cursor, Error, Span, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::Position,
        lexer::{Token, TokenKind},
    },
    Interner,
};
use bitflags::bitflags;
use std::io::{self, ErrorKind};
use std::str;
use std::{
    fmt::{self, Display, Formatter},
    io::Read,
};

#[cfg(feature = "deser")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Regex literal lexing.
///
/// Lexes Division, Assigndiv or Regex literal.
///
/// Expects: Initial '/' to already be consumed by cursor.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://www.ecma-international.org/ecma-262/#sec-literals-regular-expression-literals
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_Expressions
#[derive(Debug, Clone, Copy)]
pub(super) struct RegexLiteral;

impl<R> Tokenizer<R> for RegexLiteral {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("RegexLiteral", "Lexing");

        let mut body = Vec::new();

        // Lex RegularExpressionBody.
        loop {
            match cursor.next_byte()? {
                None => {
                    // Abrupt end.
                    return Err(Error::syntax(
                        "abrupt end on regular expression",
                        cursor.pos(),
                    ));
                }
                Some(b) => {
                    match b {
                        b'/' => break, // RegularExpressionBody finished.
                        b'\n' | b'\r' => {
                            // Not allowed in Regex literal.
                            return Err(Error::syntax(
                                "new lines are not allowed in regular expressions",
                                cursor.pos(),
                            ));
                        }
                        0xE2 if (cursor.peek_n(2)? == 0xA8_80 || cursor.peek_n(2)? == 0xA9_80) => {
                            // '\u{2028}' (e2 80 a8) and '\u{2029}' (e2 80 a9) are not allowed
                            return Err(Error::syntax(
                                "new lines are not allowed in regular expressions",
                                cursor.pos(),
                            ));
                        }
                        b'\\' => {
                            // Escape sequence
                            body.push(b'\\');
                            if let Some(sc) = cursor.next_byte()? {
                                match sc {
                                    b'\n' | b'\r' => {
                                        // Not allowed in Regex literal.
                                        return Err(Error::syntax(
                                            "new lines are not allowed in regular expressions",
                                            cursor.pos(),
                                        ));
                                    }
                                    0xE2 if (cursor.peek_n(2)? == 0xA8_80
                                        || cursor.peek_n(2)? == 0xA9_80) =>
                                    {
                                        // '\u{2028}' (e2 80 a8) and '\u{2029}' (e2 80 a9) are not allowed
                                        return Err(Error::syntax(
                                            "new lines are not allowed in regular expressions",
                                            cursor.pos(),
                                        ));
                                    }
                                    b => body.push(b),
                                }
                            } else {
                                // Abrupt end of regex.
                                return Err(Error::syntax(
                                    "abrupt end on regular expression",
                                    cursor.pos(),
                                ));
                            }
                        }
                        _ => body.push(b),
                    }
                }
            }
        }

        let mut flags = Vec::new();
        let flags_start = cursor.pos();
        cursor.take_while_ascii_pred(&mut flags, &|c: char| c.is_alphabetic())?;

        let flags_str = unsafe { str::from_utf8_unchecked(flags.as_slice()) };
        if let Ok(body_str) = str::from_utf8(body.as_slice()) {
            Ok(Token::new(
                TokenKind::regular_expression_literal(
                    interner.get_or_intern(body_str),
                    parse_regex_flags(flags_str, flags_start)?,
                ),
                Span::new(start_pos, cursor.pos()),
            ))
        } else {
            Err(Error::from(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid UTF-8 character in regular expressions",
            )))
        }
    }
}

bitflags! {
    /// Flags of a regular expression.
    #[derive(Default)]
    pub struct RegExpFlags: u8 {
        const GLOBAL = 0b0000_0001;
        const IGNORE_CASE = 0b0000_0010;
        const MULTILINE = 0b0000_0100;
        const DOT_ALL = 0b0000_1000;
        const UNICODE = 0b0001_0000;
        const STICKY = 0b0010_0000;
    }
}

pub(crate) fn parse_regex_flags(s: &str, start: Position) -> Result<RegExpFlags, Error> {
    let mut flags = RegExpFlags::default();
    for c in s.bytes() {
        let new_flag = match c {
            b'g' => RegExpFlags::GLOBAL,
            b'i' => RegExpFlags::IGNORE_CASE,
            b'm' => RegExpFlags::MULTILINE,
            b's' => RegExpFlags::DOT_ALL,
            b'u' => RegExpFlags::UNICODE,
            b'y' => RegExpFlags::STICKY,
            _ => {
                return Err(Error::syntax(
                    format!("invalid regular expression flag {}", char::from(c)),
                    start,
                ))
            }
        };

        if !flags.contains(new_flag) {
            flags.insert(new_flag);
        } else {
            return Err(Error::syntax(
                format!("invalid regular expression flag {}", char::from(c)),
                start,
            ));
        }
    }
    Ok(flags)
}

impl Display for RegExpFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use fmt::Write;

        if self.contains(Self::GLOBAL) {
            f.write_char('g')?;
        }
        if self.contains(Self::IGNORE_CASE) {
            f.write_char('i')?;
        }
        if self.contains(Self::MULTILINE) {
            f.write_char('m')?;
        }
        if self.contains(Self::DOT_ALL) {
            f.write_char('s')?;
        }
        if self.contains(Self::UNICODE) {
            f.write_char('u')?;
        }
        if self.contains(Self::STICKY) {
            f.write_char('y')?;
        }
        Ok(())
    }
}

#[cfg(feature = "deser")]
impl Serialize for RegExpFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "deser")]
impl<'de> Deserialize<'de> for RegExpFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        /// Deserializer visitor implementation for `RegExpFlags`.
        #[derive(Debug, Clone, Copy)]
        struct RegExpFlagsVisitor;

        impl<'de> Visitor<'de> for RegExpFlagsVisitor {
            type Value = RegExpFlags;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string representing JavaScript regular expression flags")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_regex_flags(value, Position::new(0, 0)).map_err(E::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_str(RegExpFlagsVisitor)
    }
}
