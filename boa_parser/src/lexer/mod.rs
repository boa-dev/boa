//! Boa's lexical analyzer(Lexer) for ECMAScript source code.
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

pub mod error;
pub mod regex;
pub mod token;

mod comment;
mod cursor;
mod identifier;
mod number;
mod operator;
mod private_identifier;
mod spread;
mod string;
mod template;

#[cfg(test)]
mod tests;

use self::{
    comment::{HashbangComment, MultiLineComment, SingleLineComment},
    cursor::Cursor,
    identifier::Identifier,
    number::NumberLiteral,
    operator::Operator,
    private_identifier::PrivateIdentifier,
    regex::RegexLiteral,
    spread::SpreadLiteral,
    string::StringLiteral,
    template::TemplateLiteral,
};
use boa_ast::{Position, Punctuator, Span};
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
    /// Sets the goal symbol for the lexer.
    pub(crate) fn set_goal(&mut self, elm: InputElement) {
        self.goal_symbol = elm;
    }

    /// Gets the goal symbol the lexer is currently using.
    pub(crate) const fn get_goal(&self) -> InputElement {
        self.goal_symbol
    }

    /// Returns if strict mode is currently active.
    pub(super) const fn strict(&self) -> bool {
        self.cursor.strict()
    }

    /// Sets the current strict mode.
    pub(super) fn set_strict(&mut self, strict: bool) {
        self.cursor.set_strict(strict);
    }

    /// Returns if module mode is currently active.
    pub(super) const fn module(&self) -> bool {
        self.cursor.module()
    }

    /// Signals that the goal symbol is a module
    pub(super) fn set_module(&mut self, module: bool) {
        self.cursor.set_module(module);
    }

    /// Creates a new lexer.
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

    /// Skips an HTML close comment (`-->`) if the `annex-b` feature is enabled.
    pub(crate) fn skip_html_close(&mut self, interner: &mut Interner) -> Result<(), Error>
    where
        R: Read,
    {
        if cfg!(not(feature = "annex-b")) || self.module() {
            return Ok(());
        }

        while self.cursor.peek_char()?.map_or(false, is_whitespace) {
            let _next = self.cursor.next_char();
        }

        if self.cursor.peek_n(3)? == [b'-', b'-', b'>'] {
            let _next = self.cursor.next_byte();
            let _next = self.cursor.next_byte();
            let _next = self.cursor.next_byte();

            let start = self.cursor.pos();
            SingleLineComment.lex(&mut self.cursor, start, interner)?;
        }

        Ok(())
    }

    /// Retrieves the next token from the lexer.
    ///
    /// # Errors
    ///
    /// Will return `Err` on invalid tokens and invalid reads of the bytes being lexed.
    // We intentionally don't implement Iterator trait as Result<Option> is cleaner to handle.
    pub(crate) fn next_no_skip(&mut self, interner: &mut Interner) -> Result<Option<Token>, Error>
    where
        R: Read,
    {
        let _timer = Profiler::global().start_event("next()", "Lexing");

        let (start, next_ch) = loop {
            let start = self.cursor.pos();
            if let Some(next_ch) = self.cursor.next_char()? {
                // Ignore whitespace
                if !is_whitespace(next_ch) {
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
                    if self.cursor.peek()?.as_ref().map(u8::is_ascii_digit) == Some(true) {
                        NumberLiteral::new(b'.').lex(&mut self.cursor, start, interner)
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
                '#' => PrivateIdentifier::new().lex(&mut self.cursor, start, interner),
                '/' => self.lex_slash_token(start, interner),
                #[cfg(feature = "annex-b")]
                '<' if !self.module() && self.cursor.peek_n(3)? == [b'!', b'-', b'-'] => {
                    let _next = self.cursor.next_byte();
                    let _next = self.cursor.next_byte();
                    let _next = self.cursor.next_byte();
                    let start = self.cursor.pos();
                    SingleLineComment.lex(&mut self.cursor, start, interner)
                }
                #[allow(clippy::cast_possible_truncation)]
                '=' | '*' | '+' | '-' | '%' | '|' | '&' | '^' | '<' | '>' | '!' | '~' | '?' => {
                    Operator::new(next_ch as u8).lex(&mut self.cursor, start, interner)
                }
                '\\' if self.cursor.peek()? == Some(b'u') => {
                    Identifier::new(c).lex(&mut self.cursor, start, interner)
                }
                _ if Identifier::is_identifier_start(c as u32) => {
                    Identifier::new(c).lex(&mut self.cursor, start, interner)
                }
                #[allow(clippy::cast_possible_truncation)]
                _ if c.is_ascii_digit() => {
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

            Ok(Some(token))
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

    /// Retrieves the next token from the lexer, skipping comments.
    ///
    /// # Errors
    ///
    /// Will return `Err` on invalid tokens and invalid reads of the bytes being lexed.
    // We intentionally don't implement Iterator trait as Result<Option> is cleaner to handle.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self, interner: &mut Interner) -> Result<Option<Token>, Error>
    where
        R: Read,
    {
        loop {
            let Some(next) = self.next_no_skip(interner)? else {
                return Ok(None)
            };

            if next.kind() != &TokenKind::Comment {
                return Ok(Some(next));
            }
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

/// Checks if a character is whitespace as per ECMAScript standards.
///
/// The Rust `char::is_whitespace` function and the ECMAScript standard use different sets of
/// characters as whitespaces:
///  * Rust uses `\p{White_Space}`,
///  * ECMAScript standard uses `\{Space_Separator}` + `\u{0009}`, `\u{000B}`, `\u{000C}`, `\u{FEFF}`
///
/// [More information](https://tc39.es/ecma262/#table-32)
const fn is_whitespace(ch: u32) -> bool {
    matches!(
        ch,
        0x0020 | 0x0009 | 0x000B | 0x000C | 0x00A0 | 0xFEFF |
            // Unicode Space_Seperator category (minus \u{0020} and \u{00A0} which are allready stated above)
            0x1680 | 0x2000..=0x200A | 0x202F | 0x205F | 0x3000
    )
}
