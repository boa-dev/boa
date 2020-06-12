use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::{Position, Span, Punctuator};
use crate::syntax::lexer::{Token, TokenKind};
use std::{
    char::{decode_utf16, from_u32},
    convert::TryFrom,
    io::{self, ErrorKind, Read},
    str,
};

/// String literal lexing.
///
/// Note: expects for the initializer `'` or `"` to already be consumed from the cursor.
#[derive(Debug, Clone, Copy)]
pub(super) struct SpreadLiteral;

impl SpreadLiteral {
    /// Creates a new string literal lexer.
    pub(super) fn new() -> Self {
        Self {}
    }
}

impl<R> Tokenizer<R> for SpreadLiteral {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        // . or ...
        match cursor.next_is('.') {
            Err(e) => {
                Err(e.into())
            },
            Ok(true) => {
                match cursor.next_is('.') {
                    Err(e) => {
                        Err(e.into())
                    },
                    Ok(true) => {
                        Ok(Token::new(Punctuator::Spread.into(), Span::new(start_pos, cursor.pos())))
                    },
                    Ok(false) => {
                        Err(Error::syntax("Expecting Token ."))
                    }
                }
            },
            Ok(false) => {
                Ok(Token::new(Punctuator::Dot.into(), Span::new(start_pos, cursor.pos())))
            }
        }
    }
}

