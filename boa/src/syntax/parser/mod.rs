//! Boa parser implementation.

mod cursor;
pub mod error;
mod expression;
mod function;
mod statement;
#[cfg(test)]
mod tests;

use self::error::{ParseError, ParseResult};
use crate::syntax::ast::{node::StatementList, Token};
use cursor::Cursor;

/// Trait implemented by parsers.
///
/// This makes it possible to abstract over the underlying implementation of a parser.
trait TokenParser: Sized {
    /// Output type for the parser.
    type Output; // = Node; waiting for https://github.com/rust-lang/rust/issues/29661

    /// Parses the token stream using the current parser.
    ///
    /// This method needs to be provided by the implementor type.
    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError>;

    /// Tries to parse the following tokens with this parser.
    ///
    /// It will return the cursor to the initial position if an error occurs during parsing.
    fn try_parse(self, cursor: &mut Cursor<'_>) -> Option<Self::Output> {
        let initial_pos = cursor.pos();
        if let Ok(node) = self.parse(cursor) {
            Some(node)
        } else {
            cursor.seek(initial_pos);
            None
        }
    }
}

/// Boolean representing if the parser should allow a `yield` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowYield(bool);

impl From<bool> for AllowYield {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `await` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowAwait(bool);

impl From<bool> for AllowAwait {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `in` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowIn(bool);

impl From<bool> for AllowIn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `return` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowReturn(bool);

impl From<bool> for AllowReturn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `default` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowDefault(bool);

impl From<bool> for AllowDefault {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

#[derive(Debug)]
pub struct Parser<'a> {
    /// Cursor in the parser, the internal structure used to read tokens.
    cursor: Cursor<'a>,
}

impl<'a> Parser<'a> {
    /// Create a new parser, using `tokens` as input
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            cursor: Cursor::new(tokens),
        }
    }

    /// Parse all expressions in the token array
    pub fn parse_all(&mut self) -> Result<StatementList, ParseError> {
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

impl TokenParser for Script {
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        if cursor.peek(0).is_some() {
            ScriptBody.parse(cursor)
        } else {
            Ok(StatementList::from(Vec::new()))
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

impl TokenParser for ScriptBody {
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        self::statement::StatementList::new(false, false, false, false).parse(cursor)
    }
}
