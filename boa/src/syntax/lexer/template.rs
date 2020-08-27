//! This module implements lexing for template literals used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
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

        let mut buf = String::new();
        loop {
            match cursor.next_char()? {
                None => {
                    return Err(Error::from(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "Unterminated template literal",
                    )));
                }
                Some('`') => break,                 // Template literal finished.
                Some(next_ch) => buf.push(next_ch), // TODO when there is an expression inside the literal
            }
        }

        Ok(Token::new(
            TokenKind::template_literal(buf),
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
