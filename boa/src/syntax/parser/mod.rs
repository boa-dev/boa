//! Boa parser implementation.

mod cursor;
pub mod error;
mod expression;
mod function;
mod statement;
#[cfg(test)]
mod tests;

pub use self::error::{ParseError, ParseResult};
use crate::syntax::{
    ast::node::StatementList,
    lexer::{Error as LexError, Position, TokenKind},
};

use cursor::Cursor;

use std::{collections::HashSet, io::Read};

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
    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError>;
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

/// Tracks all of the declared names during parsing. This is a small wrapper over two
/// `HashSet`s, which store the var delcared names, and the lexically declared names.
#[derive(Debug, Clone)]
pub struct DeclaredNames {
    vars: HashSet<Box<str>>,
    lex: HashSet<Box<str>>,
}

impl Default for DeclaredNames {
    fn default() -> Self {
        DeclaredNames {
            vars: HashSet::new(),
            lex: HashSet::new(),
        }
    }
}

impl DeclaredNames {
    /// Inserts a new variable name.
    pub fn insert_var_name(&mut self, name: &str) -> bool {
        self.vars.insert(name.into())
    }
    /// Returns an error if the a variable with the same name already has been declared.
    pub fn check_var_name(&mut self, name: &str, pos: Position) -> Result<(), ParseError> {
        if self.vars.contains(name) {
            Err(ParseError::lex(LexError::Syntax(
                format!("Redeclaration of variable `{}`", name).into(),
                pos,
            )))
        } else {
            Ok(())
        }
    }
    /// Inserts a lexically declared name.
    pub fn insert_lex_name(&mut self, name: &str) -> bool {
        self.vars.insert(name.into())
    }
    /// Returns an error if the a lexically declared name already exists.
    pub fn check_lex_name(&mut self, name: &str, pos: Position) -> Result<(), ParseError> {
        if self.vars.contains(name) {
            Err(ParseError::lex(LexError::Syntax(
                format!("Redeclaration of variable `{}`", name).into(),
                pos,
            )))
        } else {
            Ok(())
        }
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
        Script.parse(&mut self.cursor, &mut DeclaredNames::default())
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError> {
        match cursor.peek(0)? {
            Some(tok) => {
                match tok.kind() {
                    TokenKind::StringLiteral(string) if string.as_ref() == "use strict" => {
                        cursor.set_strict_mode(true);
                    }
                    _ => {}
                }
                ScriptBody.parse(cursor, env)
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError> {
        self::statement::StatementList::new(false, false, false, false, &[]).parse(cursor, env)
    }
}
