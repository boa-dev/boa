use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::{Position, Punctuator, Span};
use crate::syntax::lexer::Token;
use std::io::Read;

/// Spread literal lexing.
///
/// Note: expects for the initializer `'` or `"` to already be consumed from the cursor.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SpreadElement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax
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
        if cursor.next_is('.')? {
            if cursor.next_is('.')? {
                Ok(Token::new(
                    Punctuator::Spread.into(),
                    Span::new(start_pos, cursor.pos()),
                ))
            } else {
                Err(Error::syntax("Expecting Token ."))
            }
        } else {
            Ok(Token::new(
                Punctuator::Dot.into(),
                Span::new(start_pos, cursor.pos()),
            ))
        }
    }
}
