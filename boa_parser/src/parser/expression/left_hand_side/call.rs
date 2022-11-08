//! Call expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions
//! [spec]: https://tc39.es/ecma262/#prod-CallExpression

use super::arguments::Arguments;
use crate::{
    lexer::TokenKind,
    parser::{
        expression::{left_hand_side::template::TaggedTemplateLiteral, Expression},
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    self as ast,
    expression::{
        access::{PrivatePropertyAccess, SimplePropertyAccess},
        Call,
    },
    Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Parses a call expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CallExpression
#[derive(Debug)]
pub(super) struct CallExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    first_member_expr: ast::Expression,
}

impl CallExpression {
    /// Creates a new `CallExpression` parser.
    pub(super) fn new<Y, A>(
        allow_yield: Y,
        allow_await: A,
        first_member_expr: ast::Expression,
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            first_member_expr,
        }
    }
}

impl<R> TokenParser<R> for CallExpression
where
    R: Read,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("CallExpression", "Parsing");

        let token = cursor.peek(0, interner).or_abrupt()?;

        let mut lhs = if token.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) {
            let args =
                Arguments::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;
            Call::new(self.first_member_expr, args).into()
        } else {
            let next_token = cursor.next(interner)?.expect("token vanished");
            return Err(Error::expected(
                ["(".to_owned()],
                next_token.to_string(interner),
                next_token.span(),
                "call expression",
            ));
        };

        while let Some(tok) = cursor.peek(0, interner)? {
            let token = tok.clone();
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let args = Arguments::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    lhs = ast::Expression::from(Call::new(lhs, args));
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor.advance(interner);

                    let access = match cursor.next(interner).or_abrupt()?.kind() {
                        TokenKind::Identifier(name) => SimplePropertyAccess::new(lhs, *name).into(),
                        TokenKind::Keyword((kw, _)) => {
                            SimplePropertyAccess::new(lhs, kw.to_sym(interner)).into()
                        }
                        TokenKind::BooleanLiteral(true) => {
                            SimplePropertyAccess::new(lhs, Sym::TRUE).into()
                        }
                        TokenKind::BooleanLiteral(false) => {
                            SimplePropertyAccess::new(lhs, Sym::FALSE).into()
                        }
                        TokenKind::NullLiteral => SimplePropertyAccess::new(lhs, Sym::NULL).into(),
                        TokenKind::PrivateIdentifier(name) => {
                            cursor.push_used_private_identifier(*name, token.span().start())?;
                            PrivatePropertyAccess::new(lhs, *name).into()
                        }
                        _ => {
                            return Err(Error::expected(
                                ["identifier".to_owned()],
                                token.to_string(interner),
                                token.span(),
                                "call expression",
                            ));
                        }
                    };

                    lhs = ast::Expression::PropertyAccess(access);
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    cursor.advance(interner);
                    let idx = Expression::new(None, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(Punctuator::CloseBracket, "call expression", interner)?;
                    lhs =
                        ast::Expression::PropertyAccess(SimplePropertyAccess::new(lhs, idx).into());
                }
                TokenKind::TemplateNoSubstitution { .. } | TokenKind::TemplateMiddle { .. } => {
                    lhs = TaggedTemplateLiteral::new(
                        self.allow_yield,
                        self.allow_await,
                        tok.span().start(),
                        lhs,
                    )
                    .parse(cursor, interner)?
                    .into();
                }
                _ => break,
            }
        }
        Ok(lhs)
    }
}
