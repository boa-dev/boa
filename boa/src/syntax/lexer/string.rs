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
    char::{decode_utf16, from_u32},
    convert::TryFrom,
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

        let mut buf = String::new();
        loop {
            let next_chr_start = cursor.pos();
            let next_chr = cursor.next_char()?.ok_or_else(|| {
                Error::from(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "unterminated string literal",
                ))
            })?;

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

                    let escape = cursor.next_char()?.ok_or_else(|| {
                        Error::from(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            "unterminated escape sequence in string literal",
                        ))
                    })?;
                    if escape != '\n' {
                        let escaped_ch = match escape {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            'b' => '\x08',
                            'f' => '\x0c',
                            '0' => '\0',
                            'x' => {
                                let mut nums = [0u8; 2];
                                cursor.fill_bytes(&mut nums)?;
                                let nums = str::from_utf8(&nums).expect("non-UTF-8 bytes found");

                                let as_num = match u64::from_str_radix(&nums, 16) {
                                    Ok(v) => v,
                                    Err(_) => 0,
                                };
                                match from_u32(as_num as u32) {
                                    Some(v) => v,
                                    None => {
                                        return Err(Error::syntax(
                                            format!(
                                                "{}: {} is not a valid Unicode scalar value",
                                                cursor.pos(),
                                                as_num
                                            ),
                                            cursor.pos(),
                                        ))
                                    }
                                }
                            }
                            'u' => {
                                // There are 2 types of codepoints. Surragate codepoints and
                                // unicode codepoints. UTF-16 could be surrogate codepoints,
                                // "\uXXXX\uXXXX" which make up a single unicode codepoint. We will
                                //  need to loop to make sure we catch all UTF-16 codepoints

                                // Support \u{X..X} (Unicode Codepoint)
                                if cursor.next_is('{')? {
                                    cursor.next_char()?.expect("{ character vanished"); // Consume the '{'.

                                    // The biggest code point is 0x10FFFF
                                    // TODO: use bytes for a bit better performance (using stack)
                                    let mut code_point = String::with_capacity(6);
                                    cursor.take_until('}', &mut code_point)?;

                                    cursor.next_char()?.expect("} character vanished"); // Consume the '}'.

                                    // We know this is a single unicode codepoint, convert to u32
                                    let as_num =
                                        u32::from_str_radix(&code_point, 16).map_err(|_| {
                                            Error::syntax(
                                                "malformed Unicode character escape sequence",
                                                cursor.pos(),
                                            )
                                        })?;
                                    if as_num > 0x10_FFFF {
                                        return Err(Error::syntax("Unicode codepoint must not be greater than 0x10FFFF in escape sequence", cursor.pos()));
                                    }
                                    char::try_from(as_num).map_err(|_| {
                                        Error::syntax(
                                            "invalid Unicode escape sequence",
                                            cursor.pos(),
                                        )
                                    })?
                                } else {
                                    let mut codepoints: Vec<u16> = vec![];
                                    loop {
                                        // Collect each character after \u e.g \uD83D will give "D83D"
                                        let mut code_point = [0u8; 4];
                                        cursor.fill_bytes(&mut code_point)?;

                                        // Convert to u16
                                        let as_num = match u16::from_str_radix(
                                            str::from_utf8(&code_point)
                                                .expect("the cursor returned invalid UTF-8"),
                                            16,
                                        ) {
                                            Ok(v) => v,
                                            Err(_) => 0,
                                        };

                                        codepoints.push(as_num);

                                        // Check for another UTF-16 codepoint
                                        if cursor.next_is('\\')? && cursor.next_is('u')? {
                                            continue;
                                        }
                                        break;
                                    }

                                    // codepoints length should either be 1 (unicode codepoint) or
                                    // 2 (surrogate codepoint). Rust's decode_utf16 will deal with
                                    // it regardless
                                    // TODO: do not panic with invalid code points.
                                    decode_utf16(codepoints.iter().copied())
                                        .next()
                                        .expect("Could not get next codepoint")
                                        .expect("Could not get next codepoint")
                                }
                            }
                            '\'' | '"' | '\\' => escape,
                            ch => {
                                let details = format!(
                                    "invalid escape sequence `{}` at line {}, column {}",
                                    next_chr_start.line_number(),
                                    next_chr_start.column_number(),
                                    ch
                                );
                                return Err(Error::syntax(details, cursor.pos()));
                            }
                        };
                        buf.push(escaped_ch);
                    }
                }
                next_ch => buf.push(next_ch),
            }
        }

        Ok(Token::new(
            TokenKind::string_literal(buf),
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
