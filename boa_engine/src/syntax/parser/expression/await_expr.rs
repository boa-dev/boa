//! Await expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
//! [spec]: https://tc39.es/ecma262/#prod-AwaitExpression

use super::unary::UnaryExpression;
use crate::syntax::{
    ast::{node::AwaitExpr, Keyword},
    lexer::TokenKind,
    parser::{AllowYield, Cursor, ParseError, TokenParser},
};
use boa_interner::Interner;
use std::io::Read;

/// Parses an await expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
/// [spec]: https://tc39.es/ecma262/#prod-AwaitExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct AwaitExpression {
    allow_yield: AllowYield,
}

impl AwaitExpression {
    /// Creates a new `AwaitExpression` parser.
    pub(in crate::syntax::parser) fn new<Y>(allow_yield: Y) -> Self
    where
        Y: Into<AllowYield>,
    {
        Self {
            allow_yield: allow_yield.into(),
        }
    }
}

impl<R> TokenParser<R> for AwaitExpression
where
    R: Read,
{
    type Output = AwaitExpr;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        cursor.expect(
            TokenKind::Keyword((Keyword::Await, false)),
            "Await expression parsing",
            interner,
        )?;
        let expr = UnaryExpression::new(None, self.allow_yield, true).parse(cursor, interner)?;
        Ok(expr.into())
    }
}
