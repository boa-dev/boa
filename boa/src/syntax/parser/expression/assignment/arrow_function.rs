//! Arrow function parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
//! [spec]: https://tc39.es/ecma262/#sec-arrow-function-definitions

use super::AssignmentExpression;
use crate::{
    syntax::{
        ast::{
            node::{ArrowFunctionDecl, FormalParameter, Node, Return, StatementList},
            Punctuator, TokenKind,
        },
        parser::{
            error::{ErrorContext, ParseError, ParseResult},
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            AllowAwait, AllowIn, AllowYield, Cursor, TokenParser,
        },
    },
    BoaProfiler,
};

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

impl TokenParser for ArrowFunction {
    type Output = ArrowFunctionDecl;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ArrowFunction", "Parsing");
        let next_token = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;
        let params = if let TokenKind::Punctuator(Punctuator::OpenParen) = &next_token.kind {
            // CoverParenthesizedExpressionAndArrowParameterList
            cursor.expect(Punctuator::OpenParen, "arrow function")?;
            let params = FormalParameters::new(self.allow_yield, self.allow_await).parse(cursor)?;
            cursor.expect(Punctuator::CloseParen, "arrow function")?;
            params
        } else {
            let param = BindingIdentifier::new(self.allow_yield, self.allow_await)
                .parse(cursor)
                .context("arrow function")?;
            Box::new([FormalParameter::new(param, None, false)])
        };

        cursor.peek_expect_no_lineterminator(0)?;

        cursor.expect(Punctuator::Arrow, "arrow function")?;

        let body = ConciseBody::new(self.allow_in).parse(cursor)?;

        Ok(ArrowFunctionDecl::new(params, body))
    }
}

/// <https://tc39.es/ecma262/#prod-ConciseBody>
#[derive(Debug, Clone, Copy)]
struct ConciseBody {
    allow_in: AllowIn,
}

impl ConciseBody {
    /// Creates a new `ConcideBody` parser.
    fn new<I>(allow_in: I) -> Self
    where
        I: Into<AllowIn>,
    {
        Self {
            allow_in: allow_in.into(),
        }
    }
}

impl TokenParser for ConciseBody {
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        match cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let _ = cursor.next();
                let body = FunctionBody::new(false, false).parse(cursor)?;
                cursor.expect(Punctuator::CloseBlock, "arrow function")?;
                Ok(body)
            }
            _ => Ok(StatementList::from(vec![Return::new(
                ExpressionBody::new(self.allow_in, false).parse(cursor)?,
                None,
            )
            .into()])),
        }
    }
}

/// <https://tc39.es/ecma262/#prod-ExpressionBody>
#[derive(Debug, Clone, Copy)]
struct ExpressionBody {
    allow_in: AllowIn,
    allow_await: AllowAwait,
}

impl ExpressionBody {
    /// Creates a new `ExpressionBody` parser.
    fn new<I, A>(allow_in: I, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl TokenParser for ExpressionBody {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        AssignmentExpression::new(self.allow_in, false, self.allow_await).parse(cursor)
    }
}
