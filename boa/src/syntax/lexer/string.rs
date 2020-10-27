//! This module implements lexing for string literals used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Position, Span},
        lexer::{Token, TokenKind},
    },
};
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
enum StringTerminator {
    SingleQuote,
    DoubleQuote,
}

impl<R> Tokenizer<R> for StringLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("StringLiteral", "Lexing");

        let mut buf: Vec<u16> = Vec::new();
        loop {
            let next_chr_start = cursor.pos();
            let next_chr = unsafe {
                char::from_u32_unchecked(cursor.next_char()?.ok_or_else(|| {
                    Error::from(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "unterminated string literal",
                    ))
                })?)
            };

            match next_chr {
                '\'' if self.terminator == StringTerminator::SingleQuote => {
                    break;
                }
                '"' if self.terminator == StringTerminator::DoubleQuote => {
                    break;
                }
                '\\' => {
                    let _timer = BoaProfiler::global()
                        .start_event("StringLiteral - escape sequence", "Lexing");

                    let escape = cursor.next_byte()?.ok_or_else(|| {
                        Error::from(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            "unterminated escape sequence in string literal",
                        ))
                    })?;

                    if escape != b'\n' {
                        match escape {
                            b'n' => buf.push('\n' as u16),
                            b'r' => buf.push('\r' as u16),
                            b't' => buf.push('\t' as u16),
                            b'b' => buf.push('\x08' as u16),
                            b'f' => buf.push('\x0c' as u16),
                            b'0' => buf.push('\0' as u16),
                            b'x' => {
                                let mut code_point_utf8_bytes = [0u8; 2];
                                cursor.fill_bytes(&mut code_point_utf8_bytes)?;
                                let code_point_str = str::from_utf8(&code_point_utf8_bytes)
                                    .expect("malformed Hexadecimal character escape sequence");
                                let code_point =
                                    u16::from_str_radix(&code_point_str, 16).map_err(|_| {
                                        Error::syntax(
                                            "invalid Hexadecimal escape sequence",
                                            cursor.pos(),
                                        )
                                    })?;

                                buf.push(code_point);
                            }
                            b'u' => {
                                // Support \u{X..X} (Unicode Codepoint)
                                if cursor.next_is(b'{')? {
                                    cursor.next_byte()?.expect("{ character vanished"); // Consume the '{'.

                                    // TODO: use bytes for a bit better performance (using stack)
                                    let mut code_point_buf = Vec::with_capacity(6);
                                    cursor.take_until(b'}', &mut code_point_buf)?;

                                    cursor.next_byte()?.expect("} character vanished"); // Consume the '}'.

                                    let code_point_str =
                                        unsafe { str::from_utf8_unchecked(code_point_buf.as_slice()) };
                                    // We know this is a single unicode codepoint, convert to u32
                                    let code_point = u32::from_str_radix(&code_point_str, 16)
                                        .map_err(|_| {
                                            Error::syntax(
                                                "malformed Unicode character escape sequence",
                                                cursor.pos(),
                                            )
                                        })?;

                                    // UTF16Encoding of a numeric code point value
                                    if code_point > 0x10_FFFF {
                                        return Err(Error::syntax("Unicode codepoint must not be greater than 0x10FFFF in escape sequence", cursor.pos()));
                                    } else if code_point <= 65535 {
                                        buf.push(code_point as u16);
                                    } else {
                                        let cu1 = ((code_point - 65536) / 1024 + 0xD800) as u16;
                                        let cu2 = ((code_point - 65536) % 1024 + 0xDC00) as u16;
                                        buf.push(cu1);
                                        buf.push(cu2);
                                    }
                                } else {
                                    // Collect each character after \u e.g \uD83D will give "D83D"
                                    let mut code_point_utf8_bytes = [0u8; 4];
                                    cursor.fill_bytes(&mut code_point_utf8_bytes)?;

                                    // Convert to u16
                                    let code_point_str = str::from_utf8(&code_point_utf8_bytes)
                                        .expect("malformed Unicode character escape sequence");
                                    let code_point = u16::from_str_radix(code_point_str, 16)
                                        .map_err(|_| {
                                            Error::syntax(
                                                "invalid Unicode escape sequence",
                                                cursor.pos(),
                                            )
                                        })?;

                                    buf.push(code_point);
                                }
                            }
                            b'\'' | b'"' | b'\\' => buf.push(escape as u16),
                            ch => {
                                let details = format!(
                                    "invalid escape sequence at line {}, column {}",
                                    next_chr_start.line_number(),
                                    next_chr_start.column_number(),
                                );
                                return Err(Error::syntax(details, cursor.pos()));
                            }
                        };
                    }
                }
                next_ch => {
                    if next_ch.len_utf16() == 1 {
                        buf.push(next_ch as u16);
                    } else {
                        let mut code_point_bytes_buf = [0u16; 2];
                        let code_point_bytes = next_ch.encode_utf16(&mut code_point_bytes_buf);

                        buf.extend(code_point_bytes.iter());
                    }
                }
            }
        }

        Ok(Token::new(
            TokenKind::string_literal(String::from_utf16_lossy(buf.as_slice())),
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
