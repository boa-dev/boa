//! A lexical analyzer for JavaScript source code.
//!
//! The Lexer splits its input source code into a sequence of input elements called tokens, represented by the [Token](../ast/token/struct.Token.html) structure.
//! It also removes whitespace and comments and attaches them to the next token.

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
pub use error::Error;
pub use token::{Token, TokenKind};

use std::io::Read;

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
    fn is_whitespace(ch: char) -> bool {
        match ch {
            '\u{0020}' | '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}' |
            // Unicode Space_Seperator category (minus \u{0020} and \u{00A0} which are allready stated above)
            '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => true,
            _ => false,
        }
    }

    /// Sets the goal symbol for the lexer.
    #[inline]
    pub(crate) fn set_goal(&mut self, elm: InputElement) {
        self.goal_symbol = elm;
    }

    #[inline]
    pub(crate) fn get_goal(&self) -> InputElement {
        self.goal_symbol
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
                '/' => {
                    self.cursor.next_char()?.expect("/ token vanished"); // Consume the '/'
                    SingleLineComment.lex(&mut self.cursor, start)
                }
                '*' => {
                    self.cursor.next_char()?.expect("* token vanished"); // Consume the '*'
                    MultiLineComment.lex(&mut self.cursor, start)
                }
                ch => {
                    match self.get_goal() {
                        InputElement::Div | InputElement::TemplateTail => {
                            // Only div punctuator allowed, regex not.

                            if ch == '=' {
                                // Indicates this is an AssignDiv.
                                self.cursor.next_char()?.expect("= token vanished"); // Consume the '='
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
            Err(Error::syntax("Abrupt end: Expecting Token /,*,= or regex"))
        }
    }

    #[allow(clippy::should_implement_trait)] // We intentionally don't implement Iterator trait as Result<Option> is cleaner to handle.
    pub fn next(&mut self) -> Result<Option<Token>, Error>
    where
        R: Read,
    {
        let _timer = BoaProfiler::global().start_event("next()", "Lexing");

        let (start, next_chr) = loop {
            let start = self.cursor.pos();
            if let Some(next_chr) = self.cursor.next_char()? {
                // Ignore whitespace
                if !Self::is_whitespace(next_chr) {
                    break (start, next_chr);
                }
            } else {
                return Ok(None);
            }
        };

        // TODO, setting strict mode on/off.
        let strict_mode = false;

        let token = match next_chr {
            '\r' | '\n' | '\u{2028}' | '\u{2029}' => Ok(Token::new(
                TokenKind::LineTerminator,
                Span::new(start, self.cursor.pos()),
            )),
            '"' | '\'' => StringLiteral::new(next_chr).lex(&mut self.cursor, start),
            '`' => TemplateLiteral.lex(&mut self.cursor, start),
            _ if next_chr.is_digit(10) => {
                NumberLiteral::new(next_chr, strict_mode).lex(&mut self.cursor, start)
            }
            _ if next_chr.is_alphabetic() || next_chr == '$' || next_chr == '_' => {
                Identifier::new(next_chr).lex(&mut self.cursor, start)
            }
            ';' => Ok(Token::new(
                Punctuator::Semicolon.into(),
                Span::new(start, self.cursor.pos()),
            )),
            ':' => Ok(Token::new(
                Punctuator::Colon.into(),
                Span::new(start, self.cursor.pos()),
            )),
            '.' => SpreadLiteral::new().lex(&mut self.cursor, start),
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
            '?' => Ok(Token::new(
                Punctuator::Question.into(),
                Span::new(start, self.cursor.pos()),
            )),
            '/' => self.lex_slash_token(start),
            '=' | '*' | '+' | '-' | '%' | '|' | '&' | '^' | '<' | '>' | '!' | '~' => {
                Operator::new(next_chr).lex(&mut self.cursor, start)
            }
            _ => {
                let details = format!(
                    "Unexpected '{}' at line {}, column {}",
                    next_chr,
                    start.line_number(),
                    start.column_number()
                );
                Err(Error::syntax(details))
            }
        }?;

        if token.kind() == &TokenKind::Comment {
            // Skip comment
            self.next()
        } else {
            Ok(Some(token))
        }
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
