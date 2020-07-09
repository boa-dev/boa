use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::{Position, Span};
use crate::syntax::lexer::{Token, TokenKind};
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
        // Skip either to the end of the line or to the end of the input
        while let Some(ch) = cursor.peek()? {
            if ch == '\n' {
                break;
            } else {
                // Consume char.
                cursor.next()?.expect("Comment character vansihed");
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
        let mut ret: Option<Token> = None;
        loop {
            let pos = cursor.pos();
            if let Some(ch) = cursor.next()? {
                if ch == '*' && cursor.next_is('/')? {
                    break;
                } else if ch == '\n' {
                    ret = Some(Token::new(
                        TokenKind::LineTerminator,
                        Span::new(pos, cursor.pos()),
                    ));
                }
            } else {
                return Err(Error::syntax("unterminated multiline comment"));
            }
        }

        if let Some(ret) = ret {
            Ok(ret)
        } else {
            Ok(Token::new(
                TokenKind::Comment,
                Span::new(start_pos, cursor.pos()),
            ))
        }
    }
}
