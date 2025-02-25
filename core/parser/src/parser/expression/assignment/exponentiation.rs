//! Exponentiation operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Exponentiation
//! [spec]: https://tc39.es/ecma262/#sec-exp-operator

use crate::{
    lexer::TokenKind,
    parser::{
        expression::{unary::UnaryExpression, update::UpdateExpression},
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    source::ReadChar,
};
use boa_ast::{
    expression::operator::{binary::ArithmeticOp, Binary},
    Expression, Keyword, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;

/// Parses an exponentiation expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Arithmetic_Operators#Exponentiation
/// [spec]: https://tc39.es/ecma262/#prod-ExponentiationExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::parser::expression) struct ExponentiationExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ExponentiationExpression {
    /// Creates a new `ExponentiationExpression` parser.
    pub(in crate::parser::expression) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for ExponentiationExpression
where
    R: ReadChar,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        self.parse_boxed(cursor, interner).map(|ok| *ok)
    }

    fn parse_boxed(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> ParseResult<Box<Self::Output>> {
        let _timer = Profiler::global().start_event("ExponentiationExpression", "Parsing");

        let next = cursor.peek(0, interner).or_abrupt()?;
        match next.kind() {
            TokenKind::Keyword((Keyword::Delete | Keyword::Void | Keyword::TypeOf, _))
            | TokenKind::Punctuator(
                Punctuator::Add | Punctuator::Sub | Punctuator::Not | Punctuator::Neg,
            ) => {
                return UnaryExpression::new(self.allow_yield, self.allow_await)
                    .parse_boxed(cursor, interner);
            }
            TokenKind::Keyword((Keyword::Await, _)) if self.allow_await.0 => {
                return UnaryExpression::new(self.allow_yield, self.allow_await)
                    .parse_boxed(cursor, interner);
            }
            _ => {}
        }

        let lhs = UpdateExpression::new(self.allow_yield, self.allow_await)
            .parse_boxed(cursor, interner)?;
        if let Some(tok) = cursor.peek(0, interner)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::Exp) {
                cursor.advance(interner);
                return Ok(Box::new(
                    Binary::new_boxed(
                        ArithmeticOp::Exp.into(),
                        lhs,
                        self.parse_boxed(cursor, interner)?,
                    )
                    .into(),
                ));
            }
        }
        Ok(lhs)
    }
}
