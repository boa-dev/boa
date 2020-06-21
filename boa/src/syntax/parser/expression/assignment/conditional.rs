//! Conditional operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_Operator
//! [spec]: https://tc39.es/ecma262/#sec-conditional-operator

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::ConditionalOp, Node, Punctuator},
        parser::{
            expression::{AssignmentExpression, LogicalORExpression},
            AllowAwait, AllowIn, AllowYield, ParseResult, Parser, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Conditional expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_Operator
/// [spec]: https://tc39.es/ecma262/#prod-ConditionalExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::expression) struct ConditionalExpression {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ConditionalExpression {
    /// Creates a new `ConditionalExpression` parser.
    pub(in crate::syntax::parser::expression) fn new<I, Y, A>(
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

impl<R> TokenParser<R> for ConditionalExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, parser: &mut Parser<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("Conditional", "Parsing");
        // TODO: coalesce expression
        let lhs = LogicalORExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse(parser)?;

        if let Some(tok) = parser.next() {
            if tok.kind == TokenKind::Punctuator(Punctuator::Question) {
                let then_clause =
                    AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(parser)?;
                parser.expect(Punctuator::Colon, "conditional expression")?;

                let else_clause =
                    AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(parser)?;
                return Ok(ConditionalOp::new(lhs, then_clause, else_clause).into());
            } else {
                parser.back();
            }
        }

        Ok(lhs)
    }
}
