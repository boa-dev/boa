use super::{Cursor, Error, Tokenizer};
use crate::builtins::BigInt;
use crate::syntax::ast::{Position, Span};
use crate::syntax::lexer::token::NumericLiteral;
use crate::syntax::lexer::{Token, TokenKind};
use std::io::{self, ErrorKind, Read};
use std::str::FromStr;

/// Identifier or keyword lexing.
///
/// This currently includes boolean/NaN lexing.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]:
/// [mdn]:
#[derive(Debug, Clone, Copy)]
pub(super) struct Identifier {
    init: char,
}

impl Identifier {
    /// Creates a new identifier/keyword lexer.
    pub(super) fn new(init: char) -> Self {
        Self { init: init }
    }
}

impl<R> Tokenizer<R> for Identifier {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let mut buf = self.init.to_string();

        loop {
            match cursor.peek() {
                None => {
                    break;
                }
                Some(Ok(c)) => {
                    if c.is_alphabetic() || c.is_digit(10) || *c == '_' {
                        let ch = cursor.next().unwrap()?;
                        buf.push(ch);
                    } else {
                        break;
                    }
                }
                Some(Err(_e)) => {
                    // TODO handle error.
                }
            }
        }
        let tk = match buf.as_str() {
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            "null" => TokenKind::NullLiteral,
            "NaN" => TokenKind::NumericLiteral(NumericLiteral::Rational(f64::NAN)),
            slice => {
                if let Ok(keyword) = FromStr::from_str(slice) {
                    TokenKind::Keyword(keyword)
                } else {
                    TokenKind::identifier(slice)
                }
            }
        };

        Ok(Token::new(tk, Span::new(start_pos, cursor.pos())))
    }
}
