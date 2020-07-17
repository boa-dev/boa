use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Position, Span},
        lexer::{Token, TokenKind},
    },
};
use std::io::Read;

/// Identifier lexing.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-Identifier
/// [mdn]: https://developer.mozilla.org/en-US/docs/Glossary/Identifier
#[derive(Debug, Clone, Copy)]
pub(super) struct Identifier {
    init: char,
}

impl Identifier {
    /// Creates a new identifier/keyword lexer.
    pub(super) fn new(init: char) -> Self {
        Self { init }
    }
}

impl<R> Tokenizer<R> for Identifier {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("Identifier", "Lexing");

        let mut buf = self.init.to_string();

        while let Some(c) = cursor.peek()? {
            if c.is_alphabetic() || c.is_digit(10) || c == '_' {
                cursor
                    .next_char()?
                    .expect("Character in identifier has vanished");
                buf.push(c);
            } else {
                break;
            }
        }

        let tk = match buf.as_str() {
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            "null" => TokenKind::NullLiteral,
            slice => {
                if let Ok(keyword) = slice.parse() {
                    TokenKind::Keyword(keyword)
                } else {
                    TokenKind::identifier(slice)
                }
            }
        };

        Ok(Token::new(tk, Span::new(start_pos, cursor.pos())))
    }
}
