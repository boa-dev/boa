//! A lexical analyzer for JavaScript source code.
//!
//! This module contains the Boa lexer or tokenizer implementation.
//!
//! The Lexer splits its input source code into a sequence of input elements called tokens,
//! represented by the [Token](../ast/token/struct.Token.html) structure. It also removes
//! whitespace and comments and attaches them to the next token.
//!
//! This is tightly coupled with the parser due to the javascript goal-symbol requirements
//! as documented by the spec.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar

mod comment;
mod cursor;
pub mod error;
mod identifier;
mod number;
mod operator;
mod regex;
mod spread;
mod string;
mod template;
pub mod token;

#[cfg(test)]
mod tests;

use self::{
    comment::{MultiLineComment, SingleLineComment},
    cursor::Cursor,
    identifier::Identifier,
    number::NumberLiteral,
    operator::Operator,
    regex::RegexLiteral,
    spread::SpreadLiteral,
    string::StringLiteral,
    template::TemplateLiteral,
};
use crate::syntax::ast::{Punctuator, Span};
pub use crate::{profiler::BoaProfiler, syntax::ast::Position};
use core::convert::TryFrom;
pub use error::Error;
use std::io::Read;
pub use token::{Token, TokenKind};

trait Tokenizer<R> {
    /// Lexes the next token.
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read;
}

/// Lexer or tokenizer for the Boa JavaScript Engine.
#[derive(Debug)]
pub struct Lexer<R> {
    cursor: Cursor<R>,
    goal_symbol: InputElement,
}

impl<R> Lexer<R> {
    /// Checks if a character is whitespace as per ECMAScript standards.
    ///
    /// The Rust `char::is_whitespace` function and the ECMAScript standard use different sets of
    /// characters as whitespaces:
    ///  * Rust uses `\p{White_Space}`,
    ///  * ECMAScript standard uses `\{Space_Separator}` + `\u{0009}`, `\u{000B}`, `\u{000C}`, `\u{FEFF}`
    ///
    /// [More information](https://tc39.es/ecma262/#table-32)
    fn is_whitespace(ch: u32) -> bool {
        matches!(
            ch,
            0x0020 | 0x0009 | 0x000B | 0x000C | 0x00A0 | 0xFEFF |
            // Unicode Space_Seperator category (minus \u{0020} and \u{00A0} which are allready stated above)
            0x1680 | 0x2000..=0x200A | 0x202F | 0x205F | 0x3000
        )
    }

    /// Sets the goal symbol for the lexer.
    #[inline]
    pub(crate) fn set_goal(&mut self, elm: InputElement) {
        self.goal_symbol = elm;
    }

    /// Gets the goal symbol the lexer is currently using.
    #[inline]
    pub(crate) fn get_goal(&self) -> InputElement {
        self.goal_symbol
    }

    #[inline]
    pub(super) fn strict_mode(&self) -> bool {
        self.cursor.strict_mode()
    }

    #[inline]
    pub(super) fn set_strict_mode(&mut self, strict_mode: bool) {
        self.cursor.set_strict_mode(strict_mode)
    }

    /// Creates a new lexer.
    #[inline]
    pub fn new(reader: R) -> Self
    where
        R: Read,
    {
        Self {
            cursor: Cursor::new(reader),
            goal_symbol: Default::default(),
        }
    }

    // Handles lexing of a token starting '/' with the '/' already being consumed.
    // This could be a divide symbol or the start of a regex.
    //
    // A '/' symbol can always be a comment but if as tested above it is not then
    // that means it could be multiple different tokens depending on the input token.
    //
    // As per https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar
    pub(crate) fn lex_slash_token(&mut self, start: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("lex_slash_token", "Lexing");

        if let Some(c) = self.cursor.peek()? {
            match c {
                b'/' => {
                    self.cursor.next_byte()?.expect("/ token vanished"); // Consume the '/'
                    SingleLineComment.lex(&mut self.cursor, start)
                }
                b'*' => {
                    self.cursor.next_byte()?.expect("* token vanished"); // Consume the '*'
                    MultiLineComment.lex(&mut self.cursor, start)
                }
                ch => {
                    match self.get_goal() {
                        InputElement::Div | InputElement::TemplateTail => {
                            // Only div punctuator allowed, regex not.

                            if ch == b'=' {
                                // Indicates this is an AssignDiv.
                                self.cursor.next_byte()?.expect("= token vanished"); // Consume the '='
                                Ok(Token::new(
                                    Punctuator::AssignDiv.into(),
                                    Span::new(start, self.cursor.pos()),
                                ))
                            } else {
                                Ok(Token::new(
                                    Punctuator::Div.into(),
                                    Span::new(start, self.cursor.pos()),
                                ))
                            }
                        }
                        InputElement::RegExp | InputElement::RegExpOrTemplateTail => {
                            // Can be a regular expression.
                            RegexLiteral.lex(&mut self.cursor, start)
                        }
                    }
                }
            }
        } else {
            Err(Error::syntax(
                "Abrupt end: Expecting Token /,*,= or regex",
                start,
            ))
        }
    }

    /// Retrieves the next token from the lexer.
    // We intentionally don't implement Iterator trait as Result<Option> is cleaner to handle.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<Token>, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("next()", "Lexing");

        let (start, next_ch) = loop {
            let start = self.cursor.pos();
            if let Some(next_ch) = self.cursor.next_char()? {
                // Ignore whitespace
                if !Self::is_whitespace(next_ch) {
                    break (start, next_ch);
                }
            } else {
                return Ok(None);
            }
        };

        if let Ok(c) = char::try_from(next_ch) {
            let token = match c {
                '\r' | '\n' | '\u{2028}' | '\u{2029}' => Ok(Token::new(
                    TokenKind::LineTerminator,
                    Span::new(start, self.cursor.pos()),
                )),
                '"' | '\'' => StringLiteral::new(c).lex(&mut self.cursor, start),
                '`' => TemplateLiteral.lex(&mut self.cursor, start),
                ';' => Ok(Token::new(
                    Punctuator::Semicolon.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                ':' => Ok(Token::new(
                    Punctuator::Colon.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                '.' => {
                    if self.cursor.peek()?.map(|c| (b'0'..=b'9').contains(&c)) == Some(true) {
                        NumberLiteral::new(next_ch as u8).lex(&mut self.cursor, start)
                    } else {
                        SpreadLiteral::new().lex(&mut self.cursor, start)
                    }
                }
                '(' => Ok(Token::new(
                    Punctuator::OpenParen.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                ')' => Ok(Token::new(
                    Punctuator::CloseParen.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                ',' => Ok(Token::new(
                    Punctuator::Comma.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                '{' => Ok(Token::new(
                    Punctuator::OpenBlock.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                '}' => Ok(Token::new(
                    Punctuator::CloseBlock.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                '[' => Ok(Token::new(
                    Punctuator::OpenBracket.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                ']' => Ok(Token::new(
                    Punctuator::CloseBracket.into(),
                    Span::new(start, self.cursor.pos()),
                )),
                '/' => self.lex_slash_token(start),
                '=' | '*' | '+' | '-' | '%' | '|' | '&' | '^' | '<' | '>' | '!' | '~' | '?' => {
                    Operator::new(next_ch as u8).lex(&mut self.cursor, start)
                }
                _ if c.is_digit(10) => {
                    NumberLiteral::new(next_ch as u8).lex(&mut self.cursor, start)
                }
                _ if Identifier::is_identifier_start(c as u32) => {
                    Identifier::new(c).lex(&mut self.cursor, start)
                }
                _ => {
                    let details = format!(
                        "unexpected '{}' at line {}, column {}",
                        c,
                        start.line_number(),
                        start.column_number()
                    );
                    Err(Error::syntax(details, start))
                }
            }?;

            if token.kind() == &TokenKind::Comment {
                // Skip comment
                self.next()
            } else {
                Ok(Some(token))
            }
        } else {
            Err(Error::syntax(
                format!(
                    "unexpected utf-8 char '\\u{}' at line {}, column {}",
                    next_ch,
                    start.line_number(),
                    start.column_number()
                ),
                start,
            ))
        }
    }

    pub(crate) fn lex_template(&mut self, start: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        TemplateLiteral.lex(&mut self.cursor, start)
    }
}

/// ECMAScript goal symbols.
///
/// <https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputElement {
    Div,
    RegExp,
    RegExpOrTemplateTail,
    TemplateTail,
}

impl Default for InputElement {
    fn default() -> Self {
        InputElement::RegExp
    }
}
