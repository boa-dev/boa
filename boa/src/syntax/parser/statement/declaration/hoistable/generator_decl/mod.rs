#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::declaration::generator_decl::GeneratorDecl, Keyword, Punctuator},
    parser::{
        statement::declaration::hoistable::parse_callable_declaration, Cursor, ParseError,
        TokenParser,
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
pub(super) struct GeneratorDeclaration<const YIELD: bool, const AWAIT: bool, const DEFAULT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const DEFAULT: bool> TokenParser<R>
    for GeneratorDeclaration<YIELD, AWAIT, DEFAULT>
where
    R: Read,
{
    type Output = GeneratorDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Function, "generator declaration")?;
        cursor.expect(Punctuator::Mul, "generator declaration")?;

        let result = parse_callable_declaration::<R, YIELD, AWAIT, DEFAULT, true, false>(
            "generator declaration",
            cursor,
        )?;

        Ok(GeneratorDecl::new(result.0, result.1, result.2))
    }
}
