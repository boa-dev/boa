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
        ast::{node::ConditionalOp, Node, Punctuator, Span},
        parser::{
            expression::{AssignmentExpression, ShortCircuitExpression},
            AllowAwait, AllowIn, AllowYield, Cursor, ParseResult, TokenParser,
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

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ConditionalExpression", "Parsing");

        let lhs = ShortCircuitExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse(cursor)?;

        if let Some(tok) = cursor.peek(0)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::Question) {
                cursor.next()?.expect("? character vanished"); // Consume the token.
                let then_clause =
                    AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor)?;
                cursor.expect(Punctuator::Colon, "conditional expression")?;

                let else_clause =
                    AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor)?;

                let span = Span::new(lhs.span().start(), else_clause.span().end());
                return Ok(Node::new(
                    ConditionalOp::new(lhs, then_clause, else_clause),
                    span,
                ));
            }
        }

        Ok(lhs)
    }
}
