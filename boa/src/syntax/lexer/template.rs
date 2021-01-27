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
                    // TODO: Cook the raw string only when needed (lazy evaluation)
                    let cooked = Self::cook_template_string(&raw, start_pos, cursor.strict_mode())?;

                    return Ok(Token::new(
                        TokenKind::template_no_substitution(raw, cooked),
                        Span::new(start_pos, cursor.pos()),
                    ));
                }
                0x0024 /* $ */ if cursor.next_is(b'{')? => {
                    let raw = buf.to_string_lossy();
                    // TODO: Cook the raw string only when needed (lazy evaluation)
                    let cooked = Self::cook_template_string(&raw, start_pos, cursor.strict_mode())?;

                    return Ok(Token::new(
                        TokenKind::template_middle(raw, cooked),
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

impl TemplateLiteral {
    fn cook_template_string(
        raw: &str,
        start_pos: Position,
        is_strict_mode: bool,
    ) -> Result<String, Error> {
        let mut cursor = Cursor::with_position(raw.as_bytes(), start_pos);
        let mut buf: Vec<u16> = Vec::new();

        loop {
            let ch_start_pos = cursor.pos();
            let ch = cursor.next_char()?;

            match ch {
                Some(0x005C /* \ */) => {
                    if let Some(escape_value) =
                        StringLiteral::take_escape_sequence_or_line_continuation(
                            &mut cursor,
                            ch_start_pos,
                            is_strict_mode,
                            true,
                        )?
                    {
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

        Ok(buf.to_string_lossy())
    }
}
