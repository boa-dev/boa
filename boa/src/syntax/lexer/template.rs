//! This module implements lexing for template literals used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::lexer::string::{StringLiteral, UTF16CodeUnitsBuffer},
    syntax::{
        ast::{Position, Span},
        lexer::{Token, TokenKind},
    },
};
use std::io::{self, ErrorKind, Read};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub struct TemplateString {
    /// The start position of the template string. Used to make lexer error if `to_owned_cooked` failed.
    start_pos: Position,
    /// The template string of template literal with argument `raw` true.
    raw: Box<str>,
}

impl TemplateString {
    pub fn new<R>(raw: R, start_pos: Position) -> Self
    where
        R: Into<Box<str>>,
    {
        Self {
            start_pos,
            raw: raw.into(),
        }
    }

    /// Converts the raw template string into a mutable string slice.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-templatestrings
    pub fn as_raw(&self) -> &str {
        self.raw.as_ref()
    }

    /// Creats a new cooked template string. Returns a lexer error if it fails to cook the template string.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-templatestrings
    pub fn to_owned_cooked(&self) -> Result<Box<str>, Error> {
        let mut cursor = Cursor::with_position(self.raw.as_bytes(), self.start_pos);
        let mut buf: Vec<u16> = Vec::new();

        loop {
            let ch_start_pos = cursor.pos();
            let ch = cursor.next_char()?;

            match ch {
                Some(0x005C /* \ */) => {
                    let escape_value = StringLiteral::take_escape_sequence_or_line_continuation(
                        &mut cursor,
                        ch_start_pos,
                        true,
                        true,
                    )?;

                    if let Some(escape_value) = escape_value {
                        buf.push_code_point(escape_value);
                    }
                }
                Some(ch) => {
                    // The caller guarantees that sequences '`' and '${' never appear
                    // LineTerminatorSequence <CR> <LF> is consumed by `cursor.next_char()` and returns <LF>,
                    // which matches the TV of <CR> <LF>
                    buf.push_code_point(ch);
                }
                None => break,
            }
        }

        Ok(buf.to_string_lossy().into())
    }
}

/// Template literal lexing.
///
/// Expects: Initial ` to already be consumed by cursor.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-template-literals
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
#[derive(Debug, Clone, Copy)]
pub(super) struct TemplateLiteral;

impl<R> Tokenizer<R> for TemplateLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("TemplateLiteral", "Lexing");

        let mut buf = Vec::new();
        loop {
            let ch = cursor.next_char()?.ok_or_else(|| {
                Error::from(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "unterminated template literal",
                ))
            })?;

            match ch {
                0x0060 /* ` */ => {
                    let raw = buf.to_string_lossy();
                    let template_string = TemplateString::new(raw, start_pos);

                    return Ok(Token::new(
                        TokenKind::template_no_substitution(template_string),
                        Span::new(start_pos, cursor.pos()),
                    ));
                }
                0x0024 /* $ */ if cursor.next_is(b'{')? => {
                    let raw = buf.to_string_lossy();
                    let template_string = TemplateString::new(raw, start_pos);

                    return Ok(Token::new(
                        TokenKind::template_middle(template_string),
                        Span::new(start_pos, cursor.pos()),
                    ));
                }
                0x005C /* \ */ => {
                    let escape_ch = cursor.peek()?.ok_or_else(|| {
                        Error::from(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            "unterminated escape sequence in literal",
                        ))
                    })?;

                    buf.push(b'\\' as u16);
                    match escape_ch {
                        b'`' | b'$' | b'\\' => buf.push(cursor.next_byte()?.unwrap() as u16),
                        _ => continue,
                    }
                }
                ch => {
                    buf.push_code_point(ch);
                }
            }
        }
    }
}
