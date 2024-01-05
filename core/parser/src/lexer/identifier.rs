//! This module implements lexing for identifiers (foo, myvar, etc.) used in ECMAScript.

use crate::lexer::{
    token::ContainsEscapeSequence, Cursor, Error, StringLiteral, Token, TokenKind, Tokenizer,
};
use crate::source::ReadChar;
use boa_ast::{Keyword, Position, Span};
use boa_interner::Interner;
use boa_profiler::Profiler;

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
    pub(super) const fn new(init: char) -> Self {
        Self { init }
    }

    /// Checks if a character is `IdentifierStart` as per ECMAScript standards.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-names-and-keywords
    pub(super) fn is_identifier_start(ch: u32) -> bool {
        matches!(ch, 0x0024 /* $ */ | 0x005F /* _ */)
            || icu_properties::sets::id_start().contains32(ch)
    }

    /// Checks if a character is `IdentifierPart` as per ECMAScript standards.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-names-and-keywords
    fn is_identifier_part(ch: u32) -> bool {
        matches!(
            ch,
            0x0024 /* $ */ | 0x005F /* _ */ | 0x200C /* <ZWNJ> */ | 0x200D /* <ZWJ> */
        ) || icu_properties::sets::id_continue().contains32(ch)
    }
}

impl<R> Tokenizer<R> for Identifier {
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: ReadChar,
    {
        let _timer = Profiler::global().start_event("Identifier", "Lexing");

        let (identifier_name, contains_escaped_chars) =
            Self::take_identifier_name(cursor, start_pos, self.init)?;

        let token_kind = match identifier_name.parse() {
            Ok(Keyword::True) => {
                TokenKind::BooleanLiteral((true, ContainsEscapeSequence(contains_escaped_chars)))
            }
            Ok(Keyword::False) => {
                TokenKind::BooleanLiteral((false, ContainsEscapeSequence(contains_escaped_chars)))
            }
            Ok(Keyword::Null) => {
                TokenKind::NullLiteral(ContainsEscapeSequence(contains_escaped_chars))
            }
            Ok(keyword) => TokenKind::Keyword((keyword, contains_escaped_chars)),
            _ => TokenKind::IdentifierName((
                interner.get_or_intern(identifier_name.as_str()),
                ContainsEscapeSequence(contains_escaped_chars),
            )),
        };

        Ok(Token::new(token_kind, Span::new(start_pos, cursor.pos())))
    }
}

impl Identifier {
    pub(super) fn take_identifier_name<R>(
        cursor: &mut Cursor<R>,
        start_pos: Position,
        init: char,
    ) -> Result<(String, bool), Error>
    where
        R: ReadChar,
    {
        let _timer = Profiler::global().start_event("Identifier::take_identifier_name", "Lexing");

        let mut contains_escaped_chars = false;
        let mut identifier_name = if init == '\\' && cursor.next_if(0x75 /* u */)? {
            let ch = StringLiteral::take_unicode_escape_sequence(cursor, start_pos)?;

            if Self::is_identifier_start(ch) {
                contains_escaped_chars = true;
                String::from(
                    char::try_from(ch)
                        .expect("all identifier starts must be convertible to strings"),
                )
            } else {
                return Err(Error::Syntax("invalid identifier start".into(), start_pos));
            }
        } else {
            // The caller guarantees that `init` is a valid identifier start
            String::from(init)
        };

        loop {
            let ch = match cursor.peek_char()? {
                Some(0x005C /* \ */) if cursor.peek_n(2)?[1] == Some(0x75) /* u */ => {
                    let pos = cursor.pos();
                    let _next = cursor.next_char();
                    let _next = cursor.next_char();
                    let ch = StringLiteral::take_unicode_escape_sequence(cursor, pos)?;

                    if Self::is_identifier_part(ch) {
                        contains_escaped_chars = true;
                        ch
                    } else {
                        return Err(Error::Syntax("invalid identifier part".into(), pos));
                    }
                }
                Some(ch) if Self::is_identifier_part(ch) => {
                    cursor.next_char()?;
                    ch
                },
                _ => break,
            };

            identifier_name.push(char::try_from(ch).expect("checked character value"));
        }

        Ok((identifier_name, contains_escaped_chars))
    }
}
