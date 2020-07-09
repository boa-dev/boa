//! Assignment operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Assignment
//! [spec]: https://tc39.es/ecma262/#sec-assignment-operators

mod arrow_function;
mod conditional;
mod exponentiation;

use self::{arrow_function::ArrowFunction, conditional::ConditionalExpression};
use crate::syntax::lexer::{InputElement, TokenKind};
use crate::{
    syntax::{
        ast::{
            node::{Assign, BinOp, Node},
            Keyword, Punctuator,
        },
        parser::{AllowAwait, AllowIn, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
    },
    BoaProfiler,
};
pub(super) use exponentiation::ExponentiationExpression;

use std::io::Read;

/// Assignment expression parsing.
///
/// This can be one of the following:
///
///  - [`ConditionalExpression`](../conditional_operator/struct.ConditionalExpression.html)
///  - `YieldExpression`
///  - [`ArrowFunction`](../../function/arrow_function/struct.ArrowFunction.html)
///  - `AsyncArrowFunction`
///  - [`LeftHandSideExpression`][lhs] `=` `AssignmentExpression`
///  - [`LeftHandSideExpression`][lhs] `AssignmentOperator` `AssignmentExpression`
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Assignment
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
/// [lhs]: ../lhs_expression/struct.LeftHandSideExpression.html
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct AssignmentExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AssignmentExpression {
    /// Creates a new `AssignmentExpression` parser.
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

impl<R> TokenParser<R> for AssignmentExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("AssignmentExpression", "Parsing");
        cursor.set_goal(InputElement::Div);

        // Arrow function
        match cursor.peek()?.ok_or(ParseError::AbruptEnd)?.kind() {
            // a=>{}
            TokenKind::Identifier(_)
            | TokenKind::Keyword(Keyword::Yield)
            | TokenKind::Keyword(Keyword::Await) => {
                if cursor.peek_expect_no_lineterminator(true).is_ok() {
                    if let Some(tok) = cursor.peek_skip()? {
                        if tok.kind() == &TokenKind::Punctuator(Punctuator::Arrow) {
                            return ArrowFunction::new(
                                self.allow_in,
                                self.allow_yield,
                                self.allow_await,
                            )
                            .parse(cursor)
                            .map(Node::ArrowFunctionDecl);
                        }
                    }
                }
            }

            // (a,b)=>{}
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                if let Some(node) =
                    ArrowFunction::new(self.allow_in, self.allow_yield, self.allow_await)
                        .try_parse(cursor)
                        .map(Node::ArrowFunctionDecl)
                {
                    return Ok(node);
                }
            }

            _ => {}
        }

        cursor.set_goal(InputElement::Div);

        let mut lhs = ConditionalExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse(cursor)?;

        if let Some(tok) = cursor.peek()? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Assign) => {
                    cursor.next()?.expect("= token vanished"); // Consume the token.
                    lhs = Assign::new(lhs, self.parse(cursor)?).into();
                }
                TokenKind::Punctuator(p) if p.as_binop().is_some() => {
                    cursor.next()?.expect("Token vanished"); // Consume the token.

                    let expr = self.parse(cursor)?;
                    let binop = p.as_binop().expect("binop disappeared");
                    lhs = BinOp::new(binop, lhs, expr).into();
                }
                _ => {}
            }
        }

        Ok(lhs)
    }
}
