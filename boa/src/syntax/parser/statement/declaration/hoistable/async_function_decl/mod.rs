#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::AsyncFunctionDecl, Keyword},
    parser::{
        statement::declaration::hoistable::parse_callable_declaration, Cursor, ParseError,
        TokenParser,
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
pub(super) struct AsyncFunctionDeclaration<
    const YIELD: bool,
    const AWAIT: bool,
    const DEFAULT: bool,
>;

impl<R, const YIELD: bool, const AWAIT: bool, const DEFAULT: bool> TokenParser<R>
    for AsyncFunctionDeclaration<YIELD, AWAIT, DEFAULT>
where
    R: Read,
{
    type Output = AsyncFunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Async, "async function declaration")?;
        cursor.peek_expect_no_lineterminator(0, "async function declaration")?;
        cursor.expect(Keyword::Function, "async function declaration")?;

        let result = parse_callable_declaration::<R, YIELD, AWAIT, DEFAULT, false, true>(
            "async function declaration",
            cursor,
        )?;

        Ok(AsyncFunctionDecl::new(result.0, result.1, result.2))
    }
}
