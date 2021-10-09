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
    parser::{Cursor, ParseError, TokenParser},
};
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
pub(in crate::syntax::parser) struct AwaitExpression<const YIELD: bool>;

impl<R, const YIELD: bool> TokenParser<R> for AwaitExpression<YIELD>
where
    R: Read,
{
    type Output = AwaitExpr;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(
            TokenKind::Keyword(Keyword::Await),
            "Await expression parsing",
        )?;
        let expr = UnaryExpression::<YIELD, true>.parse(cursor)?;
        Ok(expr.into())
    }
}
