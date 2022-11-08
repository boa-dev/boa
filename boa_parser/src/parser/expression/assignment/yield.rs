//! `YieldExpression` parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
//! [spec]: https://tc39.es/ecma262/#prod-YieldExpression

use super::AssignmentExpression;
use crate::{
    lexer::TokenKind,
    parser::{AllowAwait, AllowIn, Cursor, OrAbrupt, ParseResult, TokenParser},
};
use boa_ast::{expression::Yield, Expression, Keyword, Punctuator};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// `YieldExpression` parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
/// [spec]: https://tc39.es/ecma262/#prod-YieldExpression
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct YieldExpression {
    allow_in: AllowIn,
    allow_await: AllowAwait,
}

impl YieldExpression {
    /// Creates a new `YieldExpression` parser.
    pub(in crate::parser) fn new<I, A>(allow_in: I, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for YieldExpression
where
    R: Read,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("YieldExpression", "Parsing");

        cursor.expect(
            TokenKind::Keyword((Keyword::Yield, false)),
            "yield expression",
            interner,
        )?;

        if matches!(
            cursor.peek_is_line_terminator(0, interner)?,
            Some(true) | None
        ) {
            return Ok(Yield::new(None, false).into());
        }

        let token = cursor.peek(0, interner).or_abrupt()?;
        match token.kind() {
            TokenKind::Punctuator(Punctuator::Mul) => {
                cursor.advance(interner);
                let expr = AssignmentExpression::new(None, self.allow_in, true, self.allow_await)
                    .parse(cursor, interner)?;
                Ok(Yield::new(Some(expr), true).into())
            }
            TokenKind::Identifier(_)
            | TokenKind::Punctuator(
                Punctuator::OpenParen
                | Punctuator::Add
                | Punctuator::Sub
                | Punctuator::Not
                | Punctuator::Neg
                | Punctuator::Inc
                | Punctuator::Dec
                | Punctuator::OpenBracket
                | Punctuator::OpenBlock
                | Punctuator::Div,
            )
            | TokenKind::Keyword((
                Keyword::Yield
                | Keyword::Await
                | Keyword::Delete
                | Keyword::Void
                | Keyword::TypeOf
                | Keyword::New
                | Keyword::This
                | Keyword::Function
                | Keyword::Class
                | Keyword::Async,
                _,
            ))
            | TokenKind::BooleanLiteral(_)
            | TokenKind::NullLiteral
            | TokenKind::StringLiteral(_)
            | TokenKind::TemplateNoSubstitution(_)
            | TokenKind::NumericLiteral(_)
            | TokenKind::RegularExpressionLiteral(_, _)
            | TokenKind::TemplateMiddle(_) => {
                let expr = AssignmentExpression::new(None, self.allow_in, true, self.allow_await)
                    .parse(cursor, interner)?;
                Ok(Yield::new(Some(expr), false).into())
            }
            _ => Ok(Yield::new(None, false).into()),
        }
    }
}
