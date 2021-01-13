//! Arrow function parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
//! [spec]: https://tc39.es/ecma262/#sec-arrow-function-definitions
#[cfg(test)]
mod tests;

use super::ConciseBody;
use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{
            node::{ArrowFunctionDecl, FormalParameter},
            Punctuator,
        },
        parser::{
            error::{ErrorContext, ParseError},
            function::FormalParameters,
            statement::BindingIdentifier,
            AllowAwait, AllowIn, AllowYield, Cursor, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Arrow function parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
/// [spec]: https://tc39.es/ecma262/#prod-ArrowFunction
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct ArrowFunction {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ArrowFunction {
    /// Creates a new `ArrowFunction` parser.
    pub(in crate::syntax::parser) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ArrowFunction
where
    R: Read,
{
    type Output = ArrowFunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ArrowFunction", "Parsing");
        let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        let params = if let TokenKind::Punctuator(Punctuator::OpenParen) = &next_token.kind() {
            // CoverParenthesizedExpressionAndArrowParameterList
            cursor.expect(Punctuator::OpenParen, "arrow function")?;

            let params =
                FormalParameters::new(self.allow_yield, self.allow_await, false).parse(cursor)?;
            cursor.expect(Punctuator::CloseParen, "arrow function")?;
            params
        } else {
            let param = BindingIdentifier::new(self.allow_yield, self.allow_await)
                .parse(cursor)
                .context("arrow function")?;
            Box::new([FormalParameter::new(param, None, false)])
        };

        cursor.peek_expect_no_lineterminator(0, "arrow function")?;

        cursor.expect(TokenKind::Punctuator(Punctuator::Arrow), "arrow function")?;
        let body = ConciseBody::new(self.allow_in, false).parse(cursor)?;
        Ok(ArrowFunctionDecl::new(params, body))
    }
}
