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
            expression::{AssignmentExpression, ShortCircuitExpression},
            Cursor, ParseResult, TokenParser,
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
pub(in crate::syntax::parser::expression) struct ConditionalExpression<
    const IN: bool,
    const YIELD: bool,
    const AWAIT: bool,
>;

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for ConditionalExpression<IN, YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ConditionalExpression", "Parsing");

        let lhs = ShortCircuitExpression::<IN, YIELD, AWAIT>::new().parse(cursor)?;

        if let Some(tok) = cursor.peek(0)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::Question) {
                cursor.next()?.expect("? character vanished"); // Consume the token.
                let then_clause = AssignmentExpression::<IN, YIELD, AWAIT>.parse(cursor)?;
                cursor.expect(Punctuator::Colon, "conditional expression")?;

                let else_clause = AssignmentExpression::<IN, YIELD, AWAIT>.parse(cursor)?;
                return Ok(ConditionalOp::new(lhs, then_clause, else_clause).into());
            }
        }

        Ok(lhs)
    }
}
