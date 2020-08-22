//! This module implements lexing for comments used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Position, Span},
        lexer::{Token, TokenKind},
    },
};
use std::io::Read;

/// Lexes a single line comment.
///
/// Assumes that the initial '//' is already consumed.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-comments
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar
pub(super) struct SingleLineComment;

impl<R> Tokenizer<R> for SingleLineComment {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("SingleLineComment", "Lexing");

        // Skip either to the end of the line or to the end of the input
        while let Some(ch) = cursor.peek()? {
            if ch == '\n' {
                break;
            } else {
                // Consume char.
                cursor.next_char()?.expect("Comment character vansihed");
            }
        }
        Ok(Token::new(
            TokenKind::Comment,
            Span::new(start_pos, cursor.pos()),
        ))
    }
}

/// Lexes a block (multi-line) comment.
///
/// Assumes that the initial '/*' is already consumed.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-comments
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar
pub(super) struct MultiLineComment;

impl<R> Tokenizer<R> for MultiLineComment {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("MultiLineComment", "Lexing");

        let mut new_line = false;
        loop {
            if let Some(ch) = cursor.next_char()? {
                if ch == '*' && cursor.next_is('/')? {
                    break;
                } else if ch == '\n' {
                    new_line = true;
                }
            } else {
                return Err(Error::syntax(
                    "unterminated multiline comment",
                    cursor.pos(),
                ));
            }
        }

        Ok(Token::new(
            if new_line {
                TokenKind::LineTerminator
            } else {
                TokenKind::Comment
            },
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
