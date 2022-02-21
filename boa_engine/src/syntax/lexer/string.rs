//! This module implements lexing for string literals used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::syntax::{
    ast::{Position, Span},
    lexer::{Token, TokenKind},
};
use boa_interner::Interner;
use boa_profiler::Profiler;
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
}

/// Extends a buffer type to store UTF-16 code units and convert to string.
pub(crate) trait UTF16CodeUnitsBuffer {
    /// Encodes the code point to UTF-16 code units and push to the buffer.
    fn push_code_point(&mut self, code_point: u32);

    /// Decodes the buffer into a String and replace the invalid data with the replacement character (U+FFFD).
    fn to_string_lossy(&self) -> String;
}

impl UTF16CodeUnitsBuffer for Vec<u16> {
    #[inline]
    fn push_code_point(&mut self, code_point: u32) {
        if code_point <= 65535 {
            self.push(code_point as u16);
        } else {
            let cu1 = ((code_point - 65536) / 1024 + 0xD800) as u16;
            let cu2 = ((code_point - 65536) % 1024 + 0xDC00) as u16;
            self.push(cu1);
            self.push(cu2);
        }
    }

    #[inline]
    fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.as_slice())
    }
}

impl<R> Tokenizer<R> for StringLiteral {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("StringLiteral", "Lexing");

        let (lit, span) =
            Self::take_string_characters(cursor, start_pos, self.terminator, cursor.strict_mode())?;

        Ok(Token::new(
            TokenKind::string_literal(interner.get_or_intern(lit)),
            span,
        ))
    }
}

impl StringLiteral {
    /// Checks if a character is `LineTerminator` as per ECMAScript standards.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-LineTerminator
    #[inline]
    pub(super) fn is_line_terminator(ch: u32) -> bool {
        matches!(
            ch,
            0x000A /* <LF> */ | 0x000D /* <CR> */ | 0x2028 /* <LS> */ | 0x2029 /* <PS> */
        )
    }

    #[inline]
    fn take_string_characters<R>(
        cursor: &mut Cursor<R>,
        start_pos: Position,
        terminator: StringTerminator,
        is_strict_mode: bool,
    ) -> Result<(String, Span), Error>
    where
        R: Read,
    {
        let mut buf = Vec::new();
        loop {
            let ch_start_pos = cursor.pos();
            let ch = cursor.next_char()?;

            match ch {
                Some(0x0027 /* ' */) if terminator == StringTerminator::SingleQuote => break,
                Some(0x0022 /* " */) if terminator == StringTerminator::DoubleQuote => break,
                Some(0x005C /* \ */) => {
                    let _timer =
                        Profiler::global().start_event("StringLiteral - escape sequence", "Lexing");

                    if let Some(escape_value) = Self::take_escape_sequence_or_line_continuation(
                        cursor,
                        ch_start_pos,
                        is_strict_mode,
                        false,
                    )? {
                        buf.push_code_point(escape_value);
                    }
                }
                Some(0x2028) => buf.push(0x2028 /* <LS> */),
                Some(0x2029) => buf.push(0x2029 /* <PS> */),
                Some(ch) if !Self::is_line_terminator(ch) => {
                    buf.push_code_point(ch);
                }
                _ => {
                    return Err(Error::from(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "unterminated string literal",
                    )));
                }
            }
        }

        Ok((buf.to_string_lossy(), Span::new(start_pos, cursor.pos())))
    }

    #[inline]
    pub(super) fn take_escape_sequence_or_line_continuation<R>(
        cursor: &mut Cursor<R>,
        start_pos: Position,
        is_strict_mode: bool,
        is_template_literal: bool,
    ) -> Result<Option<u32>, Error>
    where
        R: Read,
    {
        let escape_ch = cursor.next_char()?.ok_or_else(|| {
            Error::from(io::Error::new(
                ErrorKind::UnexpectedEof,
                "unterminated escape sequence in literal",
            ))
        })?;

        let escape_value = match escape_ch {
            0x0062 /* b */ => Some(0x0008 /* <BS> */),
            0x0074 /* t */ => Some(0x0009 /* <HT> */),
            0x006E /* n */ => Some(0x000A /* <LF> */),
            0x0076 /* v */ => Some(0x000B /* <VT> */),
            0x0066 /* f */ => Some(0x000C /* <FF> */),
            0x0072 /* r */ => Some(0x000D /* <CR> */),
            0x0022 /* " */ => Some(0x0022 /* " */),
            0x0027 /* ' */ => Some(0x0027 /* ' */),
            0x005C /* \ */ => Some(0x005C /* \ */),
            0x0030 /* 0 */ if cursor
                .peek()?
                .filter(|next_byte| (b'0'..=b'9').contains(next_byte))
                .is_none() =>
                Some(0x0000 /* NULL */),
            0x0078 /* x */ => {
                Some(Self::take_hex_escape_sequence(cursor, start_pos)?)
            }
            0x0075 /* u */ => {
                Some(Self::take_unicode_escape_sequence(cursor, start_pos)?)
            }
            0x0038 /* 8 */ | 0x0039 /* 9 */ => {
                // Grammar: NonOctalDecimalEscapeSequence
                if is_template_literal {
                    return Err(Error::syntax(
                        "\\8 and \\9 are not allowed in template literal",
                        start_pos,
                    ));
                } else if is_strict_mode {
                    return Err(Error::syntax(
                        "\\8 and \\9 are not allowed in strict mode",
                        start_pos,
                    ));
                }
                    Some(escape_ch)
            }
            _ if (0x0030..=0x0037 /* '0'..='7' */).contains(&escape_ch) => {
                if is_template_literal {
                    return Err(Error::syntax(
                        "octal escape sequences are not allowed in template literal",
                        start_pos,
                    ));
                }

                if is_strict_mode {
                    return Err(Error::syntax(
                        "octal escape sequences are not allowed in strict mode",
                        start_pos,
                    ));
                }

                Some(Self::take_legacy_octal_escape_sequence(
                    cursor,
                    escape_ch as u8,
                )?)
            }
            _ if Self::is_line_terminator(escape_ch) => {
                // Grammar: LineContinuation
                // Grammar: \ LineTerminatorSequence
                // LineContinuation is the empty String.
                None
            }
            _ => {
                Some(escape_ch)
            }
        };

        Ok(escape_value)
    }

    #[inline]
    pub(super) fn take_unicode_escape_sequence<R>(
        cursor: &mut Cursor<R>,
        start_pos: Position,
    ) -> Result<u32, Error>
    where
        R: Read,
    {
        // Support \u{X..X} (Unicode CodePoint)
        if cursor.next_is(b'{')? {
            // TODO: use bytes for a bit better performance (using stack)
            let mut code_point_buf = Vec::with_capacity(6);
            cursor.take_until(b'}', &mut code_point_buf)?;

            let code_point = str::from_utf8(code_point_buf.as_slice())
                .ok()
                .and_then(|code_point_str| {
                    // The `code_point_str` should represent a single unicode codepoint, convert to u32
                    u32::from_str_radix(code_point_str, 16).ok()
                })
                .ok_or_else(|| {
                    Error::syntax("malformed Unicode character escape sequence", start_pos)
                })?;

            // UTF16Encoding of a numeric code point value
            if code_point > 0x10_FFFF {
                return Err(Error::syntax(
                    "Unicode codepoint must not be greater than 0x10FFFF in escape sequence",
                    start_pos,
                ));
            }

            Ok(code_point)
        } else {
            // Grammar: Hex4Digits
            // Collect each character after \u e.g \uD83D will give "D83D"
            let mut code_point_utf8_bytes = [0u8; 4];
            cursor.fill_bytes(&mut code_point_utf8_bytes)?;

            // Convert to u16
            let code_point = str::from_utf8(&code_point_utf8_bytes)
                .ok()
                .and_then(|code_point_str| u16::from_str_radix(code_point_str, 16).ok())
                .ok_or_else(|| Error::syntax("invalid Unicode escape sequence", start_pos))?;

            Ok(u32::from(code_point))
        }
    }

    #[inline]
    fn take_hex_escape_sequence<R>(
        cursor: &mut Cursor<R>,
        start_pos: Position,
    ) -> Result<u32, Error>
    where
        R: Read,
    {
        let mut code_point_utf8_bytes = [0u8; 2];
        cursor.fill_bytes(&mut code_point_utf8_bytes)?;
        let code_point = str::from_utf8(&code_point_utf8_bytes)
            .ok()
            .and_then(|code_point_str| u16::from_str_radix(code_point_str, 16).ok())
            .ok_or_else(|| Error::syntax("invalid Hexadecimal escape sequence", start_pos))?;

        Ok(u32::from(code_point))
    }

    #[inline]
    fn take_legacy_octal_escape_sequence<R>(
        cursor: &mut Cursor<R>,
        init_byte: u8,
    ) -> Result<u32, Error>
    where
        R: Read,
    {
        // Grammar: OctalDigit
        let mut code_point = u32::from(init_byte - b'0');

        // Grammar: ZeroToThree OctalDigit
        // Grammar: FourToSeven OctalDigit
        if let Some(byte) = cursor.peek()? {
            if (b'0'..=b'7').contains(&byte) {
                let _ = cursor.next_byte()?;
                code_point = (code_point * 8) + u32::from(byte - b'0');

                if (b'0'..=b'3').contains(&init_byte) {
                    // Grammar: ZeroToThree OctalDigit OctalDigit
                    if let Some(byte) = cursor.peek()? {
                        if (b'0'..=b'7').contains(&byte) {
                            let _ = cursor.next_byte()?;
                            code_point = (code_point * 8) + u32::from(byte - b'0');
                        }
                    }
                }
            }
        }

        Ok(code_point)
    }
}
