//! This module implements lexing for template literals used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::lexer::string::{StringLiteral, StringTerminator},
    syntax::{
        ast::{Position, Span},
        lexer::{Token, TokenKind},
    },
};
use std::convert::TryFrom;
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
            let next_chr = char::try_from(cursor.next_char()?.ok_or_else(|| {
                Error::from(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "unterminated template literal",
                ))
            })?)
            .unwrap();
            match next_chr {
                '`' => {
                    let raw = String::from_utf16_lossy(buf.as_slice());
                    let (cooked, _) = StringLiteral::take_string_characters(
                        &mut Cursor::with_position(raw.as_bytes(), start_pos),
                        start_pos,
                        StringTerminator::End,
                        true,
                    )?;
                    return Ok(Token::new(
                        TokenKind::template_no_substitution(raw, cooked),
                        Span::new(start_pos, cursor.pos()),
                    ));
                }
                '$' if cursor.peek()? == Some(b'{') => {
                    let _ = cursor.next_byte()?;
                    let raw = String::from_utf16_lossy(buf.as_slice());
                    let (cooked, _) = StringLiteral::take_string_characters(
                        &mut Cursor::with_position(raw.as_bytes(), start_pos),
                        start_pos,
                        StringTerminator::End,
                        true,
                    )?;
                    return Ok(Token::new(
                        TokenKind::template_middle(raw, cooked),
                        Span::new(start_pos, cursor.pos()),
                    ));
                }
                '\\' => {
                    let escape = cursor.peek()?.ok_or_else(|| {
                        Error::from(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            "unterminated escape sequence in literal",
                        ))
                    })?;
                    buf.push('\\' as u16);
                    match escape {
                        b'`' | b'$' | b'\\' => buf.push(cursor.next_byte()?.unwrap() as u16),
                        _ => continue,
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
    }
}
