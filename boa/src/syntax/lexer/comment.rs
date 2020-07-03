use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::{Position, Span};
use crate::syntax::lexer::{Token, TokenKind};
use std::io::Read;

pub(super) struct SingleLineComment;

/// Lexes a single line comment.
///
/// Assumes that the initial '//' is already consumed.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]:
/// [mdn]:
impl<R> Tokenizer<R> for SingleLineComment {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        // Skip either to the end of the line or to the end of the input
        while let Some(ch) = cursor.next() {
            match ch {
                Err(e) => {
                    return Err(Error::IO(e));
                }
                Ok('\n') => {
                    break;
                }
                _ => {}
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
/// [spec]:
/// [mdn]:
pub(super) struct BlockComment;
impl<R> Tokenizer<R> for BlockComment {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        loop {
            if let Some(ch) = cursor.next() {
                match ch {
                    Err(e) => {
                        return Err(Error::IO(e));
                    }
                    Ok('*') => {
                        if cursor.next_is('/')? {
                            break;
                        }
                    }
                    _ => {}
                }
            } else {
                return Err(Error::syntax("unterminated multiline comment"));
            }
        }
        Ok(Token::new(
            TokenKind::Comment,
            Span::new(start_pos, cursor.pos()),
        ))
    }
}
