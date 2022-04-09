//! Async Generator Declaration parsing
//!
//! Implements `TokenParser` for `AsyncGeneratorDeclaration`on and outputs an `AsyncGeneratorDecl`
//! ast node.

#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::AsyncGeneratorDecl, Keyword, Punctuator},
    parser::{
        statement::declaration::hoistable::{parse_callable_declaration, CallableDeclaration},
        AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use boa_interner::Interner;
use std::io::Read;

/// Async Generator Declaration Parser
///
/// More information:
/// - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct AsyncGeneratorDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl AsyncGeneratorDeclaration {
    /// Creates a new `AsyncGeneratorDeclaration` parser.
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

impl CallableDeclaration for AsyncGeneratorDeclaration {
    #[inline]
    fn error_context(&self) -> &'static str {
        "async generator declaration"
    }

    #[inline]
    fn is_default(&self) -> bool {
        self.is_default.0
    }

    #[inline]
    fn name_allow_yield(&self) -> bool {
        self.allow_yield.0
    }

    #[inline]
    fn name_allow_await(&self) -> bool {
        self.allow_await.0
    }

    #[inline]
    fn parameters_allow_yield(&self) -> bool {
        true
    }

    #[inline]
    fn parameters_allow_await(&self) -> bool {
        true
    }

    #[inline]
    fn body_allow_yield(&self) -> bool {
        true
    }

    #[inline]
    fn body_allow_await(&self) -> bool {
        true
    }
}

impl<R> TokenParser<R> for AsyncGeneratorDeclaration
where
    R: Read,
{
    type Output = AsyncGeneratorDecl;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
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

        Ok(AsyncGeneratorDecl::new(result.0, result.1, result.2))
    }
}
