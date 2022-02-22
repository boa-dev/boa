//! A lexical analyzer for JavaScript source code.
//!
//! This module contains the Boa lexer or tokenizer implementation.
//!
//! The Lexer splits its input source code into a sequence of input elements called tokens,
//! represented by the [Token] structure. It also removes
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
pub mod regex;
mod spread;
mod string;
mod template;
pub mod token;

#[cfg(test)]
mod tests;

use self::{
    comment::{HashbangComment, MultiLineComment, SingleLineComment},
    cursor::Cursor,
    identifier::Identifier,
    number::NumberLiteral,
    operator::Operator,
    regex::RegexLiteral,
    spread::SpreadLiteral,
    string::StringLiteral,
    template::TemplateLiteral,
};
use crate::syntax::ast::{Position, Punctuator, Span};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

pub use self::{
    error::Error,
    token::{Token, TokenKind},
};

trait Tokenizer<R> {
    /// Lexes the next token.
    fn lex(
        &mut self,
        cursor: &mut Cursor<R>,
        start_pos: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
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
        self.cursor.set_strict_mode(strict_mode);
    }

    /// Creates a new lexer.
    #[inline]
    pub fn new(reader: R) -> Self
    where
        R: Read,
    {
        Self {
            cursor: Cursor::new(reader),
            goal_symbol: InputElement::default(),
        }
    }

    // Handles lexing of a token starting '/' with the '/' already being consumed.
    // This could be a divide symbol or the start of a regex.
    //
    // A '/' symbol can always be a comment but if as tested above it is not then
    // that means it could be multiple different tokens depending on the input token.
    //
    // As per https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar
    pub(crate) fn lex_slash_token(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("lex_slash_token", "Lexing");

        if let Some(c) = self.cursor.peek()? {
            match c {
                b'/' => {
                    self.cursor.next_byte()?.expect("/ token vanished"); // Consume the '/'
                    SingleLineComment.lex(&mut self.cursor, start, interner)
                }
                b'*' => {
                    self.cursor.next_byte()?.expect("* token vanished"); // Consume the '*'
                    MultiLineComment.lex(&mut self.cursor, start, interner)
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
                        InputElement::RegExp => {
                            // Can be a regular expression.
                            RegexLiteral.lex(&mut self.cursor, start, interner)
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
    pub fn next(&mut self, interner: &mut Interner) -> Result<Option<Token>, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("next()", "Lexing");

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

        //handle hashbang here so the below match block still throws error on
        //# if position isn't (1, 1)
        if start.column_number() == 1 && start.line_number() == 1 && next_ch == 0x23 {
            if let Some(hashbang_peek) = self.cursor.peek()? {
                if hashbang_peek == 0x21 {
                    let _token = HashbangComment.lex(&mut self.cursor, start, interner);
                    return self.next(interner);
                }
            }
        };

        if let Ok(c) = char::try_from(next_ch) {
            let token = match c {
                '\r' | '\n' | '\u{2028}' | '\u{2029}' => Ok(Token::new(
                    TokenKind::LineTerminator,
                    Span::new(start, self.cursor.pos()),
                )),
                '"' | '\'' => StringLiteral::new(c).lex(&mut self.cursor, start, interner),
                '`' => TemplateLiteral.lex(&mut self.cursor, start, interner),
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
                        NumberLiteral::new(next_ch as u8).lex(&mut self.cursor, start, interner)
                    } else {
                        SpreadLiteral::new().lex(&mut self.cursor, start, interner)
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
                '/' => self.lex_slash_token(start, interner),
                '=' | '*' | '+' | '-' | '%' | '|' | '&' | '^' | '<' | '>' | '!' | '~' | '?' => {
                    Operator::new(next_ch as u8).lex(&mut self.cursor, start, interner)
                }
                '\\' if self.cursor.peek()? == Some(b'u') => {
                    Identifier::new(c).lex(&mut self.cursor, start, interner)
                }
                _ if Identifier::is_identifier_start(c as u32) => {
                    Identifier::new(c).lex(&mut self.cursor, start, interner)
                }
                _ if c.is_digit(10) => {
                    NumberLiteral::new(next_ch as u8).lex(&mut self.cursor, start, interner)
                }
                _ => {
                    let details = format!(
                        "unexpected '{c}' at line {}, column {}",
                        start.line_number(),
                        start.column_number()
                    );
                    Err(Error::syntax(details, start))
                }
            }?;

            if token.kind() == &TokenKind::Comment {
                // Skip comment
                self.next(interner)
            } else {
                Ok(Some(token))
            }
        } else {
            Err(Error::syntax(
                format!(
                    "unexpected utf-8 char '\\u{next_ch}' at line {}, column {}",
                    start.line_number(),
                    start.column_number()
                ),
                start,
            ))
        }
    }

    /// Performs the lexing of a template literal.
    pub(crate) fn lex_template(
        &mut self,
        start: Position,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: Read,
    {
        TemplateLiteral.lex(&mut self.cursor, start, interner)
    }
}

/// ECMAScript goal symbols.
///
/// <https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputElement {
    Div,
    RegExp,
    TemplateTail,
}

impl Default for InputElement {
    fn default() -> Self {
        Self::RegExp
    }
}
