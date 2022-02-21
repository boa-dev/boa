//! Exponentiation operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Exponentiation
//! [spec]: https://tc39.es/ecma262/#sec-exp-operator

use super::ParseError;
use crate::syntax::{
    ast::{
        node::{BinOp, Node},
        op::NumOp,
        Keyword, Punctuator,
    },
    lexer::TokenKind,
    parser::{
        expression::{unary::UnaryExpression, update::UpdateExpression},
        AllowAwait, AllowYield, Cursor, ParseResult, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Parses an exponentiation expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Exponentiation
/// [spec]: https://tc39.es/ecma262/#prod-ExponentiationExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::expression) struct ExponentiationExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ExponentiationExpression {
    /// Creates a new `ExponentiationExpression` parser.
    pub(in crate::syntax::parser::expression) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

/// Checks by looking at the next token to see whether it's a unary operator or not.
fn is_unary_expression<R>(
    cursor: &mut Cursor<R>,
    interner: &mut Interner,
) -> Result<bool, ParseError>
where
    R: Read,
{
    Ok(if let Some(tok) = cursor.peek(0, interner)? {
        matches!(
            tok.kind(),
            TokenKind::Keyword(Keyword::Delete | Keyword::Void | Keyword::TypeOf)
                | TokenKind::Punctuator(
                    Punctuator::Add | Punctuator::Sub | Punctuator::Not | Punctuator::Neg
                )
        )
    } else {
        false
    })
}

impl<R> TokenParser<R> for ExponentiationExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("ExponentiationExpression", "Parsing");

        if is_unary_expression(cursor, interner)? {
            return UnaryExpression::new(self.allow_yield, self.allow_await)
                .parse(cursor, interner);
        }

        let lhs =
            UpdateExpression::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;
        if let Some(tok) = cursor.peek(0, interner)? {
            if let TokenKind::Punctuator(Punctuator::Exp) = tok.kind() {
                cursor.next(interner)?.expect("** token vanished"); // Consume the token.
                return Ok(BinOp::new(NumOp::Exp, lhs, self.parse(cursor, interner)?).into());
            }
        }
        Ok(lhs)
    }
}
