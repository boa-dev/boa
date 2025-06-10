//! Unary operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
//! [spec]: https://tc39.es/ecma262/#sec-unary-operators

use crate::{
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::{await_expr::AwaitExpression, update::UpdateExpression},
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    source::ReadChar,
    Error,
};
use boa_ast::{
    expression::{
        access::PropertyAccess,
        operator::{unary::UnaryOp, Unary},
    },
    Expression, Keyword, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;

/// Parses a unary expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct UnaryExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UnaryExpression {
    /// Creates a new `UnaryExpression` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for UnaryExpression
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
        let _timer = Profiler::global().start_event("UnaryExpression", "Parsing");

        let tok = cursor.peek(0, interner).or_abrupt()?;
        let token_start = tok.span().start();
        match tok.kind() {
            TokenKind::Keyword((Keyword::Delete | Keyword::Void | Keyword::TypeOf, true)) => Err(
                Error::general("Keyword must not contain escaped characters", token_start),
            ),
            TokenKind::Keyword((Keyword::Delete, false)) => {
                cursor.advance(interner);
                let position = cursor.peek(0, interner).or_abrupt()?.span().start();
                let target = self.parse_boxed(cursor, interner)?;

                match target.flatten() {
                    Expression::Identifier(_) if cursor.strict() => {
                        return Err(Error::lex(LexError::Syntax(
                            "cannot delete variables in strict mode".into(),
                            token_start,
                        )));
                    }
                    Expression::PropertyAccess(PropertyAccess::Private(_)) => {
                        return Err(Error::lex(LexError::Syntax(
                            "cannot delete private fields".into(),
                            position,
                        )));
                    }
                    _ => {}
                }

                Ok(Unary::new_boxed(UnaryOp::Delete, target).into())
            }
            TokenKind::Keyword((Keyword::Void, false)) => {
                cursor.advance(interner);
                Ok(Unary::new_boxed(UnaryOp::Void, self.parse_boxed(cursor, interner)?).into())
            }
            TokenKind::Keyword((Keyword::TypeOf, false)) => {
                cursor.advance(interner);
                Ok(Unary::new_boxed(UnaryOp::TypeOf, self.parse_boxed(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Add) => {
                cursor.advance(interner);
                Ok(Unary::new_boxed(UnaryOp::Plus, self.parse_boxed(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Sub) => {
                cursor.advance(interner);
                Ok(Unary::new_boxed(UnaryOp::Minus, self.parse_boxed(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Neg) => {
                cursor.advance(interner);
                Ok(Unary::new_boxed(UnaryOp::Tilde, self.parse_boxed(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Not) => {
                cursor.advance(interner);
                Ok(Unary::new_boxed(UnaryOp::Not, self.parse_boxed(cursor, interner)?).into())
            }
            TokenKind::Keyword((Keyword::Await, true)) if self.allow_await.0 => {
                Err(Error::general(
                    "Keyword 'await' must not contain escaped characters",
                    token_start,
                ))
            }
            TokenKind::Keyword((Keyword::Await, false)) if self.allow_await.0 => Ok(Box::new(
                (AwaitExpression::new(self.allow_yield).parse(cursor, interner)?).into(),
            )),
            _ => UpdateExpression::new(self.allow_yield, self.allow_await)
                .parse_boxed(cursor, interner),
        }
    }
}
