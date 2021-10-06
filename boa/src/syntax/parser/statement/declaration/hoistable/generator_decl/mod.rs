#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::declaration::generator_decl::GeneratorDecl, Keyword, Punctuator},
    parser::{
        statement::declaration::hoistable::parse_function_like_declaration, AllowAwait,
        AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use std::io::Read;

/// Generator declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function*
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct GeneratorDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl GeneratorDeclaration {
    /// Creates a new `GeneratorDeclaration` parser.
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

impl<R> TokenParser<R> for GeneratorDeclaration
where
    R: Read,
{
    type Output = GeneratorDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Function, "generator declaration")?;
        cursor.expect(Punctuator::Mul, "generator declaration")?;

        let result = parse_function_like_declaration(
            "generator declaration",
            self.is_default.0,
            self.allow_yield.0,
            self.allow_await.0,
            true,
            false,
            true,
            false,
            cursor,
        )?;

        Ok(GeneratorDecl::new(result.0, result.1, result.2))
    }
}
