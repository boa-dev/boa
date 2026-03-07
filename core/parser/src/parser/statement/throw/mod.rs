#[cfg(test)]
mod tests;

use crate::{
    parser::{AllowAwait, AllowYield, Cursor, ParseResult, TokenParser, expression::Expression},
    source::ReadChar,
};
use boa_ast::{Keyword, statement::Throw};
use boa_interner::Interner;

/// For statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw
/// [spec]: https://tc39.es/ecma262/#prod-ThrowStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct ThrowStatement<'arena> {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    _marker: std::marker::PhantomData<&'arena ()>,
}

impl ThrowStatement<'_> {
    /// Creates a new `ThrowStatement` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'arena, R> TokenParser<'arena, R> for ThrowStatement<'arena>
where
    R: ReadChar,
{
    type Output = Throw<'arena>;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect((Keyword::Throw, false), "throw statement", interner)?;

        cursor.peek_expect_no_lineterminator(0, "throw statement", interner)?;

        let expr =
            Expression::new(true, self.allow_yield, self.allow_await).parse(cursor, interner)?;

        cursor.expect_semicolon("throw statement", interner)?;

        Ok(Throw::new(expr))
    }
}
