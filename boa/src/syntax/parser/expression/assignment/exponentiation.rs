//! Exponentiation operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Exponentiation
//! [spec]: https://tc39.es/ecma262/#sec-exp-operator

use crate::{
    syntax::{
        ast::{
            node::{BinOp, Node},
            op::NumOp,
            Keyword, Punctuator, TokenKind,
        },
        parser::{
            expression::{unary::UnaryExpression, update::UpdateExpression},
            AllowAwait, AllowYield, Cursor, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

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

impl ExponentiationExpression {
    /// Checks by looking at the next token to see whether it's a unary operator or not.
    fn is_unary_expression(cursor: &mut Cursor<'_>) -> bool {
        if let Some(tok) = cursor.peek(0) {
            match tok.kind {
                TokenKind::Keyword(Keyword::Delete)
                | TokenKind::Keyword(Keyword::Void)
                | TokenKind::Keyword(Keyword::TypeOf)
                | TokenKind::Punctuator(Punctuator::Add)
                | TokenKind::Punctuator(Punctuator::Sub)
                | TokenKind::Punctuator(Punctuator::Not)
                | TokenKind::Punctuator(Punctuator::Neg) => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl TokenParser for ExponentiationExpression {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ExponentiationExpression", "Parsing");
        if Self::is_unary_expression(cursor) {
            return UnaryExpression::new(self.allow_yield, self.allow_await).parse(cursor);
        }

        let lhs = UpdateExpression::new(self.allow_yield, self.allow_await).parse(cursor)?;
        if let Some(tok) = cursor.next() {
            if let TokenKind::Punctuator(Punctuator::Exp) = tok.kind {
                return Ok(BinOp::new(NumOp::Exp, lhs, self.parse(cursor)?).into());
            } else {
                cursor.back();
            }
        }
        Ok(lhs)
    }
}
