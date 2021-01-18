//! This module implements lexing for string literals used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Position, Span},
        lexer::{Token, TokenKind},
    },
};
use core::convert::TryFrom;
use std::{
    io::{self, ErrorKind, Read},
    str,
};

/// String literal lexing.
///
/// Note: expects for the initializer `'` or `"` to already be consumed from the cursor.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-literals-string-literals
/// [mdn]: https://developer.cdn.mozilla.net/en-US/docs/Web/JavaScript/Reference/Global_Objects/String
#[derive(Debug, Clone, Copy)]
pub(super) struct StringLiteral {
    terminator: StringTerminator,
}

impl StringLiteral {
    /// Creates a new string literal lexer.
    pub(super) fn new(init: char) -> Self {
        let terminator = match init {
            '\'' => StringTerminator::SingleQuote,
            '"' => StringTerminator::DoubleQuote,
            _ => unreachable!(),
        };

        Self { terminator }
    }
}

/// Terminator for the string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StringTerminator {
    SingleQuote,
    DoubleQuote,
    End,
}

impl<R> Tokenizer<R> for StringLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("StringLiteral", "Lexing");

        let (lit, span) =
            Self::take_string_characters(cursor, start_pos, self.terminator, cursor.strict_mode())?;

        Ok(Token::new(TokenKind::string_literal(lit), span))
    }
}

impl StringLiteral {
    pub(super) fn take_string_characters<R>(
        cursor: &mut Cursor<R>,
        start_pos: Position,
        terminator: StringTerminator,
        strict_mode: bool,
    ) -> Result<(String, Span), Error>
    where
        R: Read,
    {
        let mut buf = Vec::new();
        loop {
            let next_ch = cursor.next_char()?.map(char::try_from).transpose().unwrap();

            match next_ch {
                Some('\'') if terminator == StringTerminator::SingleQuote => {
                    break;
                }
                Some('"') if terminator == StringTerminator::DoubleQuote => {
                    break;
                }
                None if terminator == StringTerminator::End => {
                    break;
                }
                Some('\\') => {
                    let _timer = BoaProfiler::global()
                        .start_event("StringLiteral - escape sequence", "Lexing");

                    let escape = cursor.peek()?.ok_or_else(|| {
                        Error::from(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            "unterminated escape sequence in literal",
                        ))
                    })?;

                    if escape <= 0x7f {
                        let _ = cursor.next_byte()?;
                        match escape {
                            b'\n' => (),
                            b'n' => buf.push('\n' as u16),
                            b'r' => buf.push('\r' as u16),
                            b't' => buf.push('\t' as u16),
                            b'b' => buf.push('\x08' as u16),
                            b'f' => buf.push('\x0c' as u16),
                            b'0' if cursor
                                .peek()?
                                .and_then(|next_byte| char::try_from(next_byte).ok())
                                .filter(|next_ch| next_ch.is_digit(10))
                                .is_none() =>
                            {
                                buf.push('\0' as u16)
                            }
                            b'x' => {
                                Self::take_hex_escape_sequence(cursor, Some(&mut buf))?;
                            }
                            b'u' => {
                                Self::take_unicode_escape_sequence(cursor, Some(&mut buf))?;
                            }
                            byte if (b'0'..b'8').contains(&byte) => {
                                Self::take_legacy_octal_escape_sequence(
                                    cursor,
                                    Some(&mut buf),
                                    strict_mode,
                                    byte,
                                )?;
                            }
                            _ => buf.push(escape as u16),
                        };
                    }
                }
                Some(next_ch) => {
                    if next_ch.len_utf16() == 1 {
                        buf.push(next_ch as u16);
                    } else {
                        let mut code_units_buf = [0u16; 2];
                        let code_units_buf = next_ch.encode_utf16(&mut code_units_buf);

                        buf.extend(code_units_buf.iter());
                    }
                }
                None => {
                    return Err(Error::from(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "unterminated string literal",
                    )));
                }
            }
        }

        Ok((
            String::from_utf16_lossy(buf.as_slice()),
            Span::new(start_pos, cursor.pos()),
        ))
    }

    #[inline]
    pub(super) fn take_unicode_escape_sequence<R>(
        cursor: &mut Cursor<R>,
        code_units_buf: Option<&mut Vec<u16>>,
    ) -> Result<u32, Error>
    where
        R: Read,
    {
        // Support \u{X..X} (Unicode CodePoint)
        if cursor.next_is(b'{')? {
            // TODO: use bytes for a bit better performance (using stack)
            let mut code_point_buf = Vec::with_capacity(6);
            cursor.take_until(b'}', &mut code_point_buf)?;

            let code_point_str = unsafe { str::from_utf8_unchecked(code_point_buf.as_slice()) };
            // We know this is a single unicode codepoint, convert to u32
            let code_point = u32::from_str_radix(&code_point_str, 16).map_err(|_| {
                Error::syntax("malformed Unicode character escape sequence", cursor.pos())
            })?;

            // UTF16Encoding of a numeric code point value
            if code_point > 0x10_FFFF {
                return Err(Error::syntax(
                    "Unicode codepoint must not be greater than 0x10FFFF in escape sequence",
                    cursor.pos(),
                ));
            } else if let Some(code_units_buf) = code_units_buf {
                if code_point <= 65535 {
                    code_units_buf.push(code_point as u16);
                } else {
                    let cu1 = ((code_point - 65536) / 1024 + 0xD800) as u16;
                    let cu2 = ((code_point - 65536) % 1024 + 0xDC00) as u16;
                    code_units_buf.push(cu1);
                    code_units_buf.push(cu2);
                }
            }

            Ok(code_point)
        } else {
            // Grammar: Hex4Digits
            // Collect each character after \u e.g \uD83D will give "D83D"
            let mut code_point_utf8_bytes = [0u8; 4];
            cursor.fill_bytes(&mut code_point_utf8_bytes)?;

            // Convert to u16
            let code_point_str = str::from_utf8(&code_point_utf8_bytes)
                .expect("malformed Unicode character escape sequence");
            let code_point = u16::from_str_radix(code_point_str, 16)
                .map_err(|_| Error::syntax("invalid Unicode escape sequence", cursor.pos()))?;

            if let Some(code_units_buf) = code_units_buf {
                code_units_buf.push(code_point);
            }

            Ok(code_point as u32)
        }
    }

    #[inline]
    fn take_hex_escape_sequence<R>(
        cursor: &mut Cursor<R>,
        code_units_buf: Option<&mut Vec<u16>>,
    ) -> Result<u32, Error>
    where
        R: Read,
    {
        let mut code_point_utf8_bytes = [0u8; 2];
        cursor.fill_bytes(&mut code_point_utf8_bytes)?;
        let code_point_str = str::from_utf8(&code_point_utf8_bytes)
            .expect("malformed Hexadecimal character escape sequence");
        let code_point = u16::from_str_radix(&code_point_str, 16)
            .map_err(|_| Error::syntax("invalid Hexadecimal escape sequence", cursor.pos()))?;

        if let Some(code_units_buf) = code_units_buf {
            code_units_buf.push(code_point);
        }

        Ok(code_point as u32)
    }

    #[inline]
    fn take_legacy_octal_escape_sequence<R>(
        cursor: &mut Cursor<R>,
        code_units_buf: Option<&mut Vec<u16>>,
        strict_mode: bool,
        init_byte: u8,
    ) -> Result<u32, Error>
    where
        R: Read,
    {
        if strict_mode {
            return Err(Error::syntax(
                "octal escape sequences are deprecated",
                cursor.pos(),
            ));
        }
        // Grammar: OctalDigit
        let mut code_point = (init_byte - b'0') as u32;

        // Grammar: ZeroToThree OctalDigit
        // Grammar: FourToSeven OctalDigit
        if let Some(byte) = cursor.peek()? {
            if (b'0'..b'8').contains(&byte) {
                let _ = cursor.next_byte()?;
                code_point = (code_point * 8) + (byte - b'0') as u32;

                if (b'0'..b'4').contains(&init_byte) {
                    // Grammar: ZeroToThree OctalDigit OctalDigit
                    if let Some(byte) = cursor.peek()? {
                        if (b'0'..b'8').contains(&byte) {
                            let _ = cursor.next_byte()?;
                            code_point = (code_point * 8) + (byte - b'0') as u32;
                        }
                    }
                }
            }
        }

        if let Some(code_units_buf) = code_units_buf {
            code_units_buf.push(code_point as u16);
        }

        Ok(code_point)
    }
}
