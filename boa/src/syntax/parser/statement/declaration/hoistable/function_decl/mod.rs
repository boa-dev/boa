#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::FunctionDecl, Keyword},
    parser::{
        statement::declaration::hoistable::parse_function_like_declaration, AllowAwait,
        AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use std::io::Read;

/// Function declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration

#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct FunctionDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl FunctionDeclaration {
    /// Creates a new `FunctionDeclaration` parser.
    pub(in crate::syntax::parser) fn new<Y, A, D>(
        allow_yield: Y,
        allow_await: A,
        is_default: D,
    ) -> Self
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

impl<R> TokenParser<R> for FunctionDeclaration
where
    R: Read,
{
    type Output = FunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Function, "function declaration")?;

        let result = parse_function_like_declaration(
            "function declaration",
            self.is_default.0,
            self.allow_yield.0,
            self.allow_await.0,
            false,
            false,
            self.allow_yield.0,
            self.allow_await.0,
            cursor,
        )?;

        Ok(FunctionDecl::new(result.0, result.1, result.2))
    }
}
