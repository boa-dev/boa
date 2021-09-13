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
mod generator_decl;

use async_function_decl::AsyncFunctionDeclaration;
pub(in crate::syntax::parser) use function_decl::FunctionDeclaration;
use generator_decl::GeneratorDeclaration;

use crate::{
    syntax::{
        ast::{Keyword, Node, Punctuator},
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

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function) => {
                let next_token = cursor.peek(1)?.ok_or(ParseError::AbruptEnd)?;
                if let TokenKind::Punctuator(Punctuator::Mul) = next_token.kind() {
                    GeneratorDeclaration::new(self.allow_yield, self.allow_await, self.is_default)
                        .parse(cursor)
                        .map(Node::from)
                } else {
                    FunctionDeclaration::new(self.allow_yield, self.allow_await, self.is_default)
                        .parse(cursor)
                        .map(Node::from)
                }
            }
            TokenKind::Keyword(Keyword::Async) => {
                AsyncFunctionDeclaration::new(self.allow_yield, self.allow_await, false)
                    .parse(cursor)
                    .map(Node::from)
            }
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}
