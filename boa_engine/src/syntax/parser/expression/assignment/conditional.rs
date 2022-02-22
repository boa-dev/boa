//! Conditional operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_Operator
//! [spec]: https://tc39.es/ecma262/#sec-conditional-operator

use crate::syntax::{
    ast::{node::ConditionalOp, Node, Punctuator},
    lexer::TokenKind,
    parser::{
        expression::{AssignmentExpression, ShortCircuitExpression},
        AllowAwait, AllowIn, AllowYield, Cursor, ParseResult, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
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

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("ConditionalExpression", "Parsing");

        let lhs = ShortCircuitExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        if let Some(tok) = cursor.peek(0, interner)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::Question) {
                cursor.next(interner)?.expect("? character vanished"); // Consume the token.
                let then_clause = AssignmentExpression::new(
                    None,
                    self.allow_in,
                    self.allow_yield,
                    self.allow_await,
                )
                .parse(cursor, interner)?;
                cursor.expect(Punctuator::Colon, "conditional expression", interner)?;

                let else_clause = AssignmentExpression::new(
                    None,
                    self.allow_in,
                    self.allow_yield,
                    self.allow_await,
                )
                .parse(cursor, interner)?;
                return Ok(ConditionalOp::new(lhs, then_clause, else_clause).into());
            }
        }

        Ok(lhs)
    }
}
