//! Async Generator Declaration parsing
//!
//! Implements `TokenParser` for `AsyncGeneratorDeclaration`on and outputs an `AsyncGeneratorDecl`
//! ast node.

#[cfg(test)]
mod tests;

use crate::parser::{
    statement::declaration::hoistable::{parse_callable_declaration, CallableDeclaration},
    AllowAwait, AllowDefault, AllowYield, Cursor, ParseResult, TokenParser,
};
use boa_ast::{function::AsyncGenerator, Keyword, Punctuator};
use boa_interner::Interner;
use std::io::Read;

/// Async Generator Declaration Parser
///
/// More information:
/// - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorDeclaration
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct AsyncGeneratorDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl AsyncGeneratorDeclaration {
    /// Creates a new `AsyncGeneratorDeclaration` parser.
    pub(in crate::parser) fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
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

impl CallableDeclaration for AsyncGeneratorDeclaration {
    fn error_context(&self) -> &'static str {
        "async generator declaration"
    }

    fn is_default(&self) -> bool {
        self.is_default.0
    }

    fn name_allow_yield(&self) -> bool {
        self.allow_yield.0
    }

    fn name_allow_await(&self) -> bool {
        self.allow_await.0
    }

    fn parameters_allow_yield(&self) -> bool {
        true
    }

    fn parameters_allow_await(&self) -> bool {
        true
    }

    fn body_allow_yield(&self) -> bool {
        true
    }

    fn body_allow_await(&self) -> bool {
        true
    }
    fn parameters_await_is_early_error(&self) -> bool {
        true
    }
    fn parameters_yield_is_early_error(&self) -> bool {
        true
    }
}

impl<R> TokenParser<R> for AsyncGeneratorDeclaration
where
    R: Read,
{
    type Output = AsyncGenerator;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect(
            (Keyword::Async, false),
            "async generator declaration",
            interner,
        )?;
        cursor.peek_expect_no_lineterminator(0, "async generator declaration", interner)?;
        cursor.expect(
            (Keyword::Function, false),
            "async generator declaration",
            interner,
        )?;
        cursor.expect(Punctuator::Mul, "async generator declaration", interner)?;

        let result = parse_callable_declaration(&self, cursor, interner)?;

        Ok(AsyncGenerator::new(
            Some(result.0),
            result.1,
            result.2,
            false,
        ))
    }
}
