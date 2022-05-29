//! This module implements lexing for regex literals used in the JavaScript programing language.

use super::{Cursor, Error, Span, Tokenizer};
use crate::syntax::{
    ast::Position,
    lexer::{Token, TokenKind},
};
use bitflags::bitflags;
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::{
    io::{self, ErrorKind, Read},
    str::{self, FromStr},
};

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
/// [spec]: https://tc39.es/ecma262/#sec-literals-regular-expression-literals
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
        let _timer = Profiler::global().start_event("RegexLiteral", "Lexing");

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
        cursor.take_while_ascii_pred(&mut flags, &char::is_alphabetic)?;

        let flags_str = unsafe { str::from_utf8_unchecked(flags.as_slice()) };
        if let Ok(body_str) = str::from_utf8(body.as_slice()) {
            Ok(Token::new(
                TokenKind::regular_expression_literal(
                    interner.get_or_intern(body_str),
                    parse_regex_flags(flags_str, flags_start, interner)?,
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
        const HAS_INDICES = 0b0100_0000;
    }
}

impl FromStr for RegExpFlags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = Self::default();
        for c in s.bytes() {
            let new_flag = match c {
                b'g' => Self::GLOBAL,
                b'i' => Self::IGNORE_CASE,
                b'm' => Self::MULTILINE,
                b's' => Self::DOT_ALL,
                b'u' => Self::UNICODE,
                b'y' => Self::STICKY,
                b'd' => Self::HAS_INDICES,
                _ => return Err(format!("invalid regular expression flag {}", char::from(c))),
            };

            if flags.contains(new_flag) {
                return Err(format!(
                    "repeated regular expression flag {}",
                    char::from(c)
                ));
            }
            flags.insert(new_flag);
        }

        Ok(flags)
    }
}

fn parse_regex_flags(s: &str, start: Position, interner: &mut Interner) -> Result<Sym, Error> {
    match RegExpFlags::from_str(s) {
        Err(message) => Err(Error::Syntax(message.into(), start)),
        Ok(flags) => Ok(interner.get_or_intern(flags.to_string())),
    }
}

impl ToString for RegExpFlags {
    fn to_string(&self) -> String {
        let mut s = String::new();
        if self.contains(Self::HAS_INDICES) {
            s.push('d');
        }
        if self.contains(Self::GLOBAL) {
            s.push('g');
        }
        if self.contains(Self::IGNORE_CASE) {
            s.push('i');
        }
        if self.contains(Self::MULTILINE) {
            s.push('m');
        }
        if self.contains(Self::DOT_ALL) {
            s.push('s');
        }
        if self.contains(Self::UNICODE) {
            s.push('u');
        }
        if self.contains(Self::STICKY) {
            s.push('y');
        }
        s
    }
}
