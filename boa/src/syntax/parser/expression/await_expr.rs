//! Await expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]:
//! [spec]:

use super::unary::UnaryExpression;

use crate::syntax::{
    ast::{node::AwaitExpr, Keyword},
    lexer::TokenKind,
    parser::{AllowYield, Cursor, ParseError, TokenParser},
};
use std::io::Read;

/// Parses a await expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]:
/// [spec]:
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

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(
            TokenKind::Keyword(Keyword::Await),
            "Await expression parsing",
        )?;
        let expr = UnaryExpression::new(self.allow_yield, true).parse(cursor)?;
        Ok(expr.into())
    }
}
