#[cfg(test)]
mod tests;

/// Async Generator Declaration Parser
///
/// Implements TokenParser for AsyncGeneratorDeclaration and outputs an
/// AsyncGeneratorDecl ast node
///
use crate::syntax::{
    ast::{node::declaration::async_generator_decl::AsyncGeneratorDecl, Punctuator},
    parser::{
        statement::declaration::hoistable::{parse_callable_declaration, CallableDeclaration},
        AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use std::io::Read;

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
}

impl<R> TokenParser<R> for AsyncGeneratorDeclaration
where
    R: Read,
{
    type Output = AsyncGeneratorDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Punctuator::Mul, "async generator declaration")?;

        let result = parse_callable_declaration(&self, cursor)?;

        Ok(AsyncGeneratorDecl::new(result.0, result.1, result.2))
    }
}
