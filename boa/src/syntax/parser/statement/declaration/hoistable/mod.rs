//! Hoistable declaration parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-HoistableDeclaration

#[cfg(test)]
mod tests;

mod async_function_decl;
mod function_decl;

use async_function_decl::AsyncFunctionDeclaration;
use function_decl::FunctionDeclaration;

use crate::{
    syntax::{
        ast::{Keyword, Node},
        lexer::TokenKind,
        parser::{
            AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};
use std::io::Read;

/// Hoistable declaration parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct HoistableDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl HoistableDeclaration {
    /// Creates a new `HoistableDeclaration` parser.
    pub(super) fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_default: is_default.into(),
        }
    }
}

impl<R> TokenParser<R> for HoistableDeclaration
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("HoistableDeclaration", "Parsing");
        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        let start = tok.span().start();

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function) => {
                FunctionDeclaration::new(self.allow_yield, self.allow_await, self.is_default)
                    .parse(cursor)
                    .map(|(kind, span)| Node::new(kind, span))
            }
            TokenKind::Keyword(Keyword::Async) => {
                AsyncFunctionDeclaration::new(self.allow_yield, self.allow_await, false)
                    .parse(cursor)
                    .map(|(kind, span)| Node::new(kind, span))
            }
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}
