//! This module implements lexing for identifiers (foo, myvar, etc.) used in the JavaScript programing language.

use super::{Cursor, Error, Tokenizer};
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Keyword, Position, Span},
        lexer::{Token, TokenKind},
    },
};
use core::convert::TryFrom;
use std::io::Read;
use std::str;

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
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("Identifier", "Lexing");

        let mut init_buf = [0u8; 4];
        let mut buf = Vec::new();
        self.init.encode_utf8(&mut init_buf);
        buf.extend(init_buf.iter().take(self.init.len_utf8()));

        cursor.take_while_char_pred(&mut buf, &|c: u32| {
            if let Ok(c) = char::try_from(c) {
                c.is_alphabetic() || c.is_digit(10) || c == '_'
            } else {
                false
            }
        })?;

        let token_str = unsafe { str::from_utf8_unchecked(buf.as_slice()) };
        let tk = match token_str {
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            "null" => TokenKind::NullLiteral,
            slice => {
                if let Ok(keyword) = slice.parse() {
                    if cursor.strict_mode() && keyword == Keyword::With {
                        return Err(Error::Syntax(
                            "using 'with' statement not allowed in strict mode".into(),
                            start_pos,
                        ));
                    }
                    TokenKind::Keyword(keyword)
                } else {
                    if cursor.strict_mode() && STRICT_FORBIDDEN_IDENTIFIERS.contains(&slice) {
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
