#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{node::AsyncFunctionExpr, Keyword, Punctuator},
        lexer::TokenKind,
        parser::{
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Async Function expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/async_function
/// [spec]: https://tc39.es/ecma262/#prod-AsyncFunctionExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct AsyncFunctionExpression {
    allow_yield: AllowYield,
}

impl AsyncFunctionExpression {
    /// Creates a new `AsyncFunctionExpression` parser.
    pub(super) fn new<Y>(allow_yield: Y) -> Self
    where
        Y: Into<AllowYield>,
    {
        Self {
            allow_yield: allow_yield.into(),
        }
    }
}

impl<R> TokenParser<R> for AsyncFunctionExpression
where
    R: Read,
{
    type Output = AsyncFunctionExpr;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("AsyncFunctionExpression", "Parsing");
        cursor.peek_expect_no_lineterminator(0, "async function expression")?;
        cursor.expect(Keyword::Function, "async function expression")?;

        let tok = cursor.peek(0)?;

        let name = if let Some(token) = tok {
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => None,
                _ => Some(BindingIdentifier::new(self.allow_yield, true).parse(cursor)?),
            }
        } else {
            return Err(ParseError::AbruptEnd);
        };

        cursor.expect(Punctuator::OpenParen, "async function expression")?;

        let params = FormalParameters::new(false, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "async function expression")?;
        cursor.expect(Punctuator::OpenBlock, "async function expression")?;

        let body = FunctionBody::new(false, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "async function expression")?;

        Ok(AsyncFunctionExpr::new(name, params, body))
    }
}
