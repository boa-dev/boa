//! Function expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#prod-FunctionExpression

use crate::{
    syntax::{
        ast::{node::FunctionExpr, Punctuator, Keyword},
        parser::{
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            Cursor, ParseError, TokenParser,
        },
        lexer::TokenKind,
    },
    BoaProfiler,
};

use std::io::Read;

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

impl<R> TokenParser<R> for FunctionExpression
where
    R: Read,
{
    type Output = FunctionExpr;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("FunctionExpression", "Parsing");

        let name = if let Some(token) = cursor.peek(false)? {
            match token.kind() {
                TokenKind::Identifier(ref s) => {
                    Some(BindingIdentifier::new(false, false).parse(cursor)?)
                }
                TokenKind::Keyword(k @ Keyword::Yield) => {
                    Some(BindingIdentifier::new(false, false).parse(cursor)?)
                }
                TokenKind::Keyword(k @ Keyword::Await) => {
                    Some(BindingIdentifier::new(false, false).parse(cursor)?)
                }
                _ => None,
            }
        } else {
            None
        };

        cursor.expect(Punctuator::OpenParen, "function expression", false)?;

        let params = FormalParameters::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "function expression", false)?;
        cursor.expect(Punctuator::OpenBlock, "function expression", false)?;

        let body = FunctionBody::new(false, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "function expression", false)?;

        Ok(FunctionExpr::new(name, params, body))
    }
}
