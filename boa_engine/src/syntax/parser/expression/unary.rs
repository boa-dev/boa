//! Unary operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
//! [spec]: https://tc39.es/ecma262/#sec-unary-operators

use crate::syntax::{
    ast::{
        expression::{
            operator::{unary::op::UnaryOp, Unary},
            Identifier,
        },
        Expression, Keyword, Punctuator,
    },
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::{await_expr::AwaitExpression, update::UpdateExpression},
        AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Parses a unary expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct UnaryExpression {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UnaryExpression {
    /// Creates a new `UnaryExpression` parser.
    pub(in crate::syntax::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for UnaryExpression
where
    R: Read,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("UnaryExpression", "Parsing");

        let tok = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let token_start = tok.span().start();
        match tok.kind() {
            TokenKind::Keyword((Keyword::Delete | Keyword::Void | Keyword::TypeOf, true)) => Err(
                ParseError::general("Keyword must not contain escaped characters", token_start),
            ),
            TokenKind::Keyword((Keyword::Delete, false)) => {
                cursor.next(interner)?.expect("Delete keyword vanished");
                let position = cursor
                    .peek(0, interner)?
                    .ok_or(ParseError::AbruptEnd)?
                    .span()
                    .start();
                let val = self.parse(cursor, interner)?;

                match val {
                    Expression::Identifier(_) if cursor.strict_mode() => {
                        return Err(ParseError::lex(LexError::Syntax(
                            "Delete <variable> statements not allowed in strict mode".into(),
                            token_start,
                        )));
                    }
                    Expression::PrivatePropertyAccess(_) => {
                        return Err(ParseError::general(
                            "private fields can not be deleted",
                            position,
                        ));
                    }
                    _ => {}
                }

                Ok(Unary::new(UnaryOp::Delete, val).into())
            }
            TokenKind::Keyword((Keyword::Void, false)) => {
                cursor.next(interner)?.expect("Void keyword vanished"); // Consume the token.
                Ok(Unary::new(UnaryOp::Void, self.parse(cursor, interner)?).into())
            }
            TokenKind::Keyword((Keyword::TypeOf, false)) => {
                cursor.next(interner)?.expect("TypeOf keyword vanished"); // Consume the token.
                Ok(Unary::new(UnaryOp::TypeOf, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Add) => {
                cursor.next(interner)?.expect("+ token vanished"); // Consume the token.
                Ok(Unary::new(UnaryOp::Plus, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Sub) => {
                cursor.next(interner)?.expect("- token vanished"); // Consume the token.
                Ok(Unary::new(UnaryOp::Minus, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Neg) => {
                cursor.next(interner)?.expect("~ token vanished"); // Consume the token.
                Ok(Unary::new(UnaryOp::Tilde, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Not) => {
                cursor.next(interner)?.expect("! token vanished"); // Consume the token.
                Ok(Unary::new(UnaryOp::Not, self.parse(cursor, interner)?).into())
            }
            TokenKind::Keyword((Keyword::Await, true)) if self.allow_await.0 => {
                Err(ParseError::general(
                    "Keyword 'await' must not contain escaped characters",
                    token_start,
                ))
            }
            TokenKind::Keyword((Keyword::Await, false)) if self.allow_await.0 => {
                Ok((AwaitExpression::new(self.allow_yield).parse(cursor, interner)?).into())
            }
            _ => UpdateExpression::new(self.name, self.allow_yield, self.allow_await)
                .parse(cursor, interner),
        }
    }
}
