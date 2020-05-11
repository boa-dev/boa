//! Arrow function parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
//! [spec]: https://tc39.es/ecma262/#sec-arrow-function-definitions

use super::AssignmentExpression;
use crate::syntax::{
    ast::{
        node::{FormalParameter, Node},
        punc::Punctuator,
        token::TokenKind,
    },
    parser::{
        function::{FormalParameters, FunctionBody},
        statement::BindingIdentifier,
        AllowAwait, AllowIn, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
    },
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
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let next_token = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;
        let params = if let TokenKind::Punctuator(Punctuator::OpenParen) = &next_token.kind {
            // CoverParenthesizedExpressionAndArrowParameterList
            cursor.expect(Punctuator::OpenParen, "arrow function")?;
            let params = FormalParameters::new(self.allow_yield, self.allow_await).parse(cursor)?;
            cursor.expect(Punctuator::CloseParen, "arrow function")?;
            params.into_boxed_slice()
        } else {
            let param = BindingIdentifier::new(self.allow_yield, self.allow_await)
                .parse(cursor)
                .map_err(|e| match e {
                    ParseError::Expected(mut exp, tok, _) => {
                        exp.push(Punctuator::OpenParen.into());
                        ParseError::Expected(exp, tok, "arrow function")
                    }
                    e => e,
                })?;
            Box::new([FormalParameter {
                init: None,
                name: param,
                is_rest_param: false,
            }])
        };

        cursor.peek_expect_no_lineterminator(0, "arrow function")?;

        cursor.expect(Punctuator::Arrow, "arrow function")?;

        let body = ConciseBody::new(self.allow_in).parse(cursor)?;

        Ok(Node::arrow_function_decl(params, body))
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
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        match cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let _ = cursor.next();
                let body = FunctionBody::new(false, false)
                    .parse(cursor)
                    .map(Node::statement_list)?;
                cursor.expect(Punctuator::CloseBlock, "arrow function")?;
                Ok(body)
            }
            _ => Ok(Node::return_node(
                ExpressionBody::new(self.allow_in, false).parse(cursor)?,
            )),
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
