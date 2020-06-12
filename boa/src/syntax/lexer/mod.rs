//! A lexical analyzer for JavaScript source code.
//!
//! The Lexer splits its input source code into a sequence of input elements called tokens, represented by the [Token](../ast/token/struct.Token.html) structure.
//! It also removes whitespace and comments and attaches them to the next token.

#[macro_use]
mod comment;
mod cursor;
pub mod error;

#[macro_use]
mod string;
pub mod token;

#[macro_use]
mod template;

mod number;

mod operator;

mod spread;

mod identifier;

// Temporary disabled while lexer in progress.
// #[cfg(test)]
// mod tests;

pub use self::error::Error;

use self::{
    cursor::Cursor, identifier::Identifier, number::NumberLiteral, string::StringLiteral,
    template::TemplateLiteral, spread::SpreadLiteral, comment::Comment, operator::Operator,
};
use crate::syntax::ast::{Position, Span, Punctuator};
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
    fn is_whitespace(ch: char) -> bool {
        match ch {
            '\u{0020}' | '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}' |
            // Unicode Space_Seperator category (minus \u{0020} and \u{00A0} which are allready stated above)
            '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => true,
            _ => false,
        }
    }

    /// Sets the goal symbol for the lexer.
    pub(crate) fn set_goal(&mut self, elm: InputElement) {
        self.goal_symbol = elm;
    }
}

impl<R> Lexer<R>
where
    R: Read,
{
    /// Creates a new lexer.
    #[inline]
    pub fn new(reader: R) -> Self {
        Self {
            cursor: Cursor::new(reader),
            goal_symbol: Default::default(),
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
        InputElement::Div
        // Decided on InputElementDiv as default for now based on documentation from
        // <https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar>
    }
}

impl<R> Iterator for Lexer<R>
where
    R: Read,
{
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let (start, next_chr) = loop {
            let start = self.cursor.pos();
            let next_chr = match self.cursor.next()? {
                Ok(c) => c,
                Err(e) => return Some(Err(e.into())),
            };

            // Ignore whitespace
            if !Self::is_whitespace(next_chr) {
                break (start, next_chr);
            }
        };

        let token = match next_chr {
            '\r' | '\n' | '\u{2028}' | '\u{2029}' => Ok(Token::new(
                TokenKind::LineTerminator,
                Span::new(start, self.cursor.pos()),
            )),
            '"' | '\'' => StringLiteral::new(next_chr).lex(&mut self.cursor, start),
            template_match!() => TemplateLiteral::new().lex(&mut self.cursor, start),
            _ if next_chr.is_digit(10) => NumberLiteral::new(next_chr).lex(&mut self.cursor, start),
            _ if next_chr.is_alphabetic() || next_chr == '$' || next_chr == '_' => {
                Identifier::new(next_chr).lex(&mut self.cursor, start)
            }
            ';' => Ok(Token::new(Punctuator::Semicolon.into(), Span::new(start, self.cursor.pos()))),
            ':' => Ok(Token::new(Punctuator::Colon.into(), Span::new(start, self.cursor.pos()))),
            '.' => SpreadLiteral::new().lex(&mut self.cursor, start),
            '(' => Ok(Token::new(Punctuator::OpenParen.into(), Span::new(start, self.cursor.pos()))),
            ')' => Ok(Token::new(Punctuator::CloseParen.into(), Span::new(start, self.cursor.pos()))),
            ',' => Ok(Token::new(Punctuator::Comma.into(), Span::new(start, self.cursor.pos()))),
            '{' => Ok(Token::new(Punctuator::OpenBlock.into(), Span::new(start, self.cursor.pos()))),
            '}' => Ok(Token::new(Punctuator::CloseBlock.into(), Span::new(start, self.cursor.pos()))),
            '[' => Ok(Token::new(Punctuator::OpenBracket.into(), Span::new(start, self.cursor.pos()))),
            ']' => Ok(Token::new(Punctuator::CloseBracket.into(), Span::new(start, self.cursor.pos()))),
            '?' => Ok(Token::new(Punctuator::Question.into(), Span::new(start, self.cursor.pos()))),
            '/' => Comment::new().lex(&mut self.cursor, start),
            '*' | '+' | '-' | '%' | '|' | '&' | '^' | '=' | '<' | '>' | '!' | '~' => Operator::new(next_chr).lex(&mut self.cursor, start),
            
            _ => unimplemented!(),
        };

        Some(token)
    }
}

// impl<R> Tokenizer<R> for Lexer<R> {
//     fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> io::Result<Token>
//     where
//         R: Read,
//     {

//     }
// }

// Temporarily moved.
use crate::syntax::ast::Keyword;

#[test]
fn check_single_line_comment() {
    let s1 = "var \n//This is a comment\ntrue";
    let mut lexer = Lexer::new(s1.as_bytes());

    assert_eq!(
        lexer.next().unwrap().unwrap().kind,
        TokenKind::Keyword(Keyword::Var)
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().kind,
        TokenKind::LineTerminator
    );
    assert_eq!(
        lexer.next().unwrap().unwrap().kind,
        TokenKind::BooleanLiteral(true)
    );
    assert!(lexer.next().is_none());
}
