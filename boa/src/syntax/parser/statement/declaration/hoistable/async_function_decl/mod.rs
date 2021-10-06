#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::AsyncFunctionDecl, Keyword},
    parser::{
        statement::declaration::hoistable::parse_function_like_declaration, AllowAwait,
        AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use std::io::Read;

/// Async Function declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
/// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#prod-AsyncFunctionDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct AsyncFunctionDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl AsyncFunctionDeclaration {
    /// Creates a new `FunctionDeclaration` parser.
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

impl<R> TokenParser<R> for AsyncFunctionDeclaration
where
    R: Read,
{
    type Output = AsyncFunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Async, "async function declaration")?;
        cursor.peek_expect_no_lineterminator(0, "async function declaration")?;
        cursor.expect(Keyword::Function, "async function declaration")?;

        let result = parse_function_like_declaration(
            "async function declaration",
            self.is_default.0,
            self.allow_yield.0,
            self.allow_await.0,
            false,
            true,
            false,
            true,
            cursor,
        )?;

        Ok(AsyncFunctionDecl::new(result.0, result.1, result.2))
    }
}
