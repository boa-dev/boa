//! Async Arrow function parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]:
//! [spec]: https://tc39.es/ecma262/#prod-AsyncArrowFunction
#[cfg(test)]
mod tests;

use super::ConciseBody;
use crate::{
    syntax::{
        ast::{
            node::{AsyncArrowFunctionDecl, FormalParameter},
            Keyword, Punctuator,
        },
        lexer::TokenKind,
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
/// [mdn]:
/// [spec]: https://tc39.es/ecma262/#prod-AsyncArrowFunction
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct AsyncArrowFunction {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AsyncArrowFunction {
    /// Creates a new `AsyncArrowFunction` parser.
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

impl<R> TokenParser<R> for AsyncArrowFunction
where
    R: Read,
{
    type Output = AsyncArrowFunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("AsyncArrowFunction", "Parsing");
        cursor.expect(
            TokenKind::Keyword(Keyword::Async),
            "async arrow function parsing",
        )?;
        let token = cursor.peek_expect_no_lineterminator(0, "async arrow function parsing")?;

        let params = if let TokenKind::Punctuator(Punctuator::OpenParen) = &token.kind() {
            // CoverCallExpressionAndAsyncArrowHead
            cursor.expect(Punctuator::OpenParen, "async arrow function")?;

            let params = FormalParameters::new(false, true, true).parse(cursor)?;
            cursor.expect(Punctuator::CloseParen, "async arrow function")?;
            params
        } else {
            // AsyncArrowBindingIdentifier
            let param = BindingIdentifier::new(self.allow_yield, true)
                .parse(cursor)
                .context("async arrow function")?;
            Box::new([FormalParameter::new(param, None, false)])
        };

        cursor.peek_expect_no_lineterminator(0, "async arrow function")?;
        cursor.expect(
            TokenKind::Punctuator(Punctuator::Arrow),
            "async arrow function",
        )?;
        let body = ConciseBody::new(self.allow_in, true).parse(cursor)?;
        Ok(AsyncArrowFunctionDecl::new(params, body))
    }
}
