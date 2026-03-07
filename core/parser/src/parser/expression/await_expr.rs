//! Await expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
//! [spec]: https://tc39.es/ecma262/#prod-AwaitExpression

use super::unary::UnaryExpression;
use crate::{
    lexer::TokenKind,
    parser::{AllowYield, Cursor, ParseResult, TokenParser},
    source::ReadChar,
};
use boa_ast::{Keyword, Span, Spanned, expression::Await};
use boa_interner::Interner;

/// Parses an await expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
/// [spec]: https://tc39.es/ecma262/#prod-AwaitExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct AwaitExpression<'arena> {
    allow_yield: AllowYield,
    _marker: std::marker::PhantomData<&'arena ()>,
}

impl AwaitExpression<'_> {
    /// Creates a new `AwaitExpression` parser.
    pub(in crate::parser) fn new<Y>(allow_yield: Y) -> Self
    where
        Y: Into<AllowYield>,
    {
        Self {
            allow_yield: allow_yield.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'arena, R> TokenParser<'arena, R> for AwaitExpression<'arena>
where
    R: ReadChar,
{
    type Output = Await<'arena>;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let await_span_start = cursor
            .expect(
                TokenKind::Keyword((Keyword::Await, false)),
                "Await expression parsing",
                interner,
            )?
            .span()
            .start();

        let expr = UnaryExpression::new(self.allow_yield, true)
            .parse(cursor, interner)?
            .try_into_expression()?;
        let expr_span_end = expr.span().end();

        Ok(Await::new(
            expr.into(),
            Span::new(await_span_start, expr_span_end),
        ))
    }
}
