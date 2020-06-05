//! Conditional operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_Operator
//! [spec]: https://tc39.es/ecma262/#sec-conditional-operator

use crate::{
    syntax::{
        ast::{Node, Punctuator, TokenKind},
        parser::{
            expression::{AssignmentExpression, LogicalORExpression},
            AllowAwait, AllowIn, AllowYield, Cursor, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

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

impl TokenParser for ConditionalExpression {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("Conditional", "Parsing");
        // TODO: coalesce expression
        let lhs = LogicalORExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse(cursor)?;

        if let Some(tok) = cursor.next() {
            if tok.kind == TokenKind::Punctuator(Punctuator::Question) {
                let then_clause =
                    AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor)?;
                cursor.expect(Punctuator::Colon, "conditional expression")?;

                let else_clause =
                    AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await)
                        .parse(cursor)?;
                return Ok(Node::conditional_op(lhs, then_clause, else_clause));
            } else {
                cursor.back();
            }
        }

        Ok(lhs)
    }
}
