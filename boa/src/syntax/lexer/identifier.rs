//! This module implements lexing for identifiers (foo, myvar, etc.) used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Keyword, Position, Span},
        lexer::{Token, TokenKind},
    },
};
use std::io::Read;

const STRICT_FORBIDDEN_IDENTIFIERS: [&str; 11] = [
    "eval",
    "arguments",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
];

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
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        strict_mode: bool,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("Identifier", "Lexing");

        let mut buf = self.init.to_string();

        cursor.take_while_pred(&mut buf, &|c: char| {
            c.is_alphabetic() || c.is_digit(10) || c == '_'
        })?;

        let tk = match buf.as_str() {
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            "null" => TokenKind::NullLiteral,
            slice => {
                if let Ok(keyword) = slice.parse() {
                    if strict_mode && keyword == Keyword::With {
                        return Err(Error::Syntax(
                            "using 'with' statement not allowed in strict mode".into(),
                            start_pos,
                        ));
                    }
                    TokenKind::Keyword(keyword)
                } else {
                    if strict_mode && STRICT_FORBIDDEN_IDENTIFIERS.contains(&slice) {
                        return Err(Error::Syntax(
                            format!(
                                "using future reserved keyword '{}' not allowed in strict mode",
                                slice
                            )
                            .into(),
                            start_pos,
                        ));
                    }
                    TokenKind::identifier(slice)
                }
            }
        };

        Ok(Token::new(tk, Span::new(start_pos, cursor.pos())))
    }
}
