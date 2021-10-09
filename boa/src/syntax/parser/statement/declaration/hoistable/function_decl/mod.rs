#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::FunctionDecl, Keyword},
    parser::{
        statement::declaration::hoistable::parse_callable_declaration, Cursor, ParseError,
        TokenParser,
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
pub(in crate::syntax::parser) struct FunctionDeclaration<
    const YIELD: bool,
    const AWAIT: bool,
    const DEFAULT: bool,
>;

impl<R, const YIELD: bool, const AWAIT: bool, const DEFAULT: bool> TokenParser<R>
    for FunctionDeclaration<YIELD, AWAIT, DEFAULT>
where
    R: Read,
{
    type Output = FunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Function, "function declaration")?;

        let result = parse_callable_declaration::<R, YIELD, AWAIT, DEFAULT, false, false>(
            "function declaration",
            cursor,
        )?;

        Ok(FunctionDecl::new(result.0, result.1, result.2))
    }
}
