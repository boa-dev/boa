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
/// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#prod-AsyncFunctionExpression
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
        cursor.expect_no_skip_lineterminator(Keyword::Function, "async function expression")?;

        let tok = cursor.peek(0)?;

        let name = if let Some(token) = tok {
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => None,
                _ => Some(BindingIdentifier::new(self.allow_yield, true).parse(cursor)?),
            }
        } else {
            return Err(ParseError::AbruptEnd);
        };

        cursor.expect(Punctuator::OpenParen, "function expression")?;

        let params = FormalParameters::new(!self.allow_yield.0, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "function expression")?;
        cursor.expect(Punctuator::OpenBlock, "function expression")?;

        let body = FunctionBody::new(!self.allow_yield.0, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "function expression")?;

        Ok(AsyncFunctionExpr::new(name, params, body))
    }
}
