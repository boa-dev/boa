//! Function expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#prod-FunctionExpression

use crate::syntax::{
    ast::{node::FunctionExpr, Punctuator},
    parser::{
        function::{FormalParameters, FunctionBody},
        statement::BindingIdentifier,
        Cursor, ParseError, TokenParser,
    },
};

/// Function expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
/// [spec]: https://tc39.es/ecma262/#prod-FunctionExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct FunctionExpression;

impl TokenParser for FunctionExpression {
    type Output = FunctionExpr;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let name = BindingIdentifier::new(false, false).try_parse(cursor);

        cursor.expect(Punctuator::OpenParen, "function expression")?;

        let params = FormalParameters::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "function expression")?;
        cursor.expect(Punctuator::OpenBlock, "function expression")?;

        let body = FunctionBody::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "function expression")?;

        Ok(FunctionExpr::new(name, params, body))
    }
}
