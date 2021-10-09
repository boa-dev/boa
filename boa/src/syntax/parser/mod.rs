//! Boa parser implementation.

mod cursor;
pub mod error;
mod expression;
mod function;
mod statement;
#[cfg(test)]
mod tests;

pub use self::error::{ParseError, ParseResult};
use crate::syntax::{ast::node::StatementList, lexer::TokenKind};

use cursor::Cursor;

use std::io::Read;

/// Trait implemented by parsers.
///
/// This makes it possible to abstract over the underlying implementation of a parser.
trait TokenParser<R>: Sized
where
    R: Read,
{
    /// Output type for the parser.
    type Output; // = Node; waiting for https://github.com/rust-lang/rust/issues/29661

    /// Parses the token stream using the current parser.
    ///
    /// This method needs to be provided by the implementor type.
    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError>;
}

/// Boolean representing if a production has the attribute `Yield`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IsYield(bool);

impl From<bool> for IsYield {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if a production has the attribute `Await`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IsAwait(bool);

impl From<bool> for IsAwait {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if a production has the attribute `In`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IsIn(bool);

impl From<bool> for IsIn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if a production has the attribute `Return`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IsReturn(bool);

impl From<bool> for IsReturn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if a production has the attribute `Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IsDefault(bool);

impl From<bool> for IsDefault {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

#[derive(Debug)]
pub struct Parser<R> {
    /// Cursor of the parser, pointing to the lexer and used to get tokens for the parser.
    cursor: Cursor<R>,
}

impl<R> Parser<R> {
    pub fn new(reader: R, strict_mode: bool) -> Self
    where
        R: Read,
    {
        let mut cursor = Cursor::new(reader);
        cursor.set_strict_mode(strict_mode);

        Self { cursor }
    }

    pub fn parse_all(&mut self) -> Result<StatementList, ParseError>
    where
        R: Read,
    {
        Script.parse(&mut self.cursor)
    }
}

/// Parses a full script.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Script
#[derive(Debug, Clone, Copy)]
pub struct Script;

impl<R> TokenParser<R> for Script
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        match cursor.peek(0)? {
            Some(tok) => {
                let mut strict = false;
                match tok.kind() {
                    TokenKind::StringLiteral(string) if string.as_ref() == "use strict" => {
                        cursor.set_strict_mode(true);
                        strict = true;
                    }
                    _ => {}
                }
                let mut statement_list = ScriptBody.parse(cursor)?;
                statement_list.set_strict(strict);
                Ok(statement_list)
            }
            None => Ok(StatementList::from(Vec::new())),
        }
    }
}

/// Parses a script body.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ScriptBody
#[derive(Debug, Clone, Copy)]
pub struct ScriptBody;

impl<R> TokenParser<R> for ScriptBody
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        self::statement::StatementList::<false, false, false, false>::new(&[]).parse(cursor)
    }
}
