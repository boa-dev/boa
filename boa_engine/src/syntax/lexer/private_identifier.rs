//! This module implements lexing for private identifiers (#foo, #myvar, etc.) used in the JavaScript programing language.

use super::{identifier::Identifier, Cursor, Error, Tokenizer};
use crate::syntax::{
    ast::{Position, Span},
    lexer::{Token, TokenKind},
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Private Identifier lexing.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-PrivateIdentifier
#[derive(Debug, Clone, Copy)]
pub(super) struct PrivateIdentifier;

impl PrivateIdentifier {
    /// Creates a new private identifier lexer.
    pub(super) fn new() -> Self {
        Self
    }
}

impl<R> Tokenizer<R> for PrivateIdentifier {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("PrivateIdentifier", "Lexing");

        if let Some(next_ch) = cursor.next_char()? {
            if let Ok(c) = char::try_from(next_ch) {
                match c {
                    '\\' if cursor.peek()? == Some(b'u') => {
                        let (name, _) = Identifier::take_identifier_name(cursor, start_pos, c)?;
                        Ok(Token::new(
                            TokenKind::PrivateIdentifier(interner.get_or_intern(&name)),
                            Span::new(start_pos, cursor.pos()),
                        ))
                    }
                    _ if Identifier::is_identifier_start(c as u32) => {
                        let (name, _) = Identifier::take_identifier_name(cursor, start_pos, c)?;
                        Ok(Token::new(
                            TokenKind::PrivateIdentifier(interner.get_or_intern(&name)),
                            Span::new(start_pos, cursor.pos()),
                        ))
                    }
                    _ => Err(Error::syntax(
                        "Abrupt end: Expecting private identifier",
                        start_pos,
                    )),
                }
            } else {
                Err(Error::syntax(
                    format!(
                        "unexpected utf-8 char '\\u{next_ch}' at line {}, column {}",
                        start_pos.line_number(),
                        start_pos.column_number()
                    ),
                    start_pos,
                ))
            }
        } else {
            Err(Error::syntax(
                "Abrupt end: Expecting private identifier",
                start_pos,
            ))
        }
    }
}
