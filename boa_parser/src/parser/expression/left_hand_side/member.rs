//! Member expression parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-MemberExpression

use super::arguments::Arguments;
use crate::{
    lexer::{token::ContainsEscapeSequence, InputElement, TokenKind},
    parser::{
        expression::{
            left_hand_side::template::TaggedTemplateLiteral, primary::PrimaryExpression, Expression,
        },
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use ast::function::PrivateName;
use boa_ast::{
    self as ast,
    expression::{
        access::{
            PrivatePropertyAccess, PropertyAccessField, SimplePropertyAccess, SuperPropertyAccess,
        },
        Call, Identifier, New,
    },
    Keyword, Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Parses a member expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-MemberExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct MemberExpression {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl MemberExpression {
    /// Creates a new `MemberExpression` parser.
    pub(super) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for MemberExpression
where
    R: Read,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("MemberExpression", "Parsing");

        cursor.set_goal(InputElement::RegExp);

        let token = cursor.peek(0, interner).or_abrupt()?;
        let position = token.span().start();
        let mut lhs = match token.kind() {
            TokenKind::Keyword((Keyword::New | Keyword::Super | Keyword::Import, true)) => {
                return Err(Error::general(
                    "keyword must not contain escaped characters",
                    token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Import, false)) => {
                cursor.advance(interner);

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::Dot),
                    "import.meta",
                    interner,
                )?;

                let token = cursor.next(interner).or_abrupt()?;

                match token.kind() {
                    TokenKind::IdentifierName((Sym::META, ContainsEscapeSequence(ces))) => {
                        if *ces {
                            return Err(Error::general(
                                "`import.meta` cannot contain escaped characters",
                                token.span().start(),
                            ));
                        }
                    }
                    _ => {
                        return Err(Error::expected(
                            ["property `meta`".into()],
                            token.to_string(interner),
                            token.span(),
                            "import.meta",
                        ));
                    }
                }

                if !cursor.module() {
                    return Err(Error::general(
                        "invalid `import.meta` expression outside a module",
                        position,
                    ));
                }

                ast::Expression::ImportMeta
            }
            TokenKind::Keyword((Keyword::New, false)) => {
                cursor.advance(interner);

                let lhs_new_target = if cursor.next_if(Punctuator::Dot, interner)?.is_some() {
                    let token = cursor.next(interner).or_abrupt()?;
                    match token.kind() {
                        TokenKind::IdentifierName((Sym::TARGET, ContainsEscapeSequence(true))) => {
                            return Err(Error::general(
                                "'new.target' must not contain escaped characters",
                                token.span().start(),
                            ));
                        }
                        TokenKind::IdentifierName((Sym::TARGET, ContainsEscapeSequence(false))) => {
                            ast::Expression::NewTarget
                        }
                        _ => {
                            return Err(Error::general(
                                "unexpected private identifier",
                                token.span().start(),
                            ));
                        }
                    }
                } else {
                    let lhs_inner = self.parse(cursor, interner)?;
                    let args = match cursor.peek(0, interner)? {
                        Some(next)
                            if next.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) =>
                        {
                            Arguments::new(self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?
                        }
                        _ => Box::new([]),
                    };
                    let call_node = Call::new(lhs_inner, args);

                    ast::Expression::from(New::from(call_node))
                };
                lhs_new_target
            }
            TokenKind::Keyword((Keyword::Super, _)) => {
                cursor.advance(interner);
                let token = cursor.next(interner).or_abrupt()?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Dot) => {
                        let token = cursor.next(interner).or_abrupt()?;
                        let field = match token.kind() {
                            TokenKind::IdentifierName((name, _)) => {
                                SuperPropertyAccess::new(PropertyAccessField::from(*name))
                            }
                            TokenKind::Keyword((kw, _)) => {
                                SuperPropertyAccess::new(kw.to_sym().into())
                            }
                            TokenKind::BooleanLiteral(true) => {
                                SuperPropertyAccess::new(Sym::TRUE.into())
                            }
                            TokenKind::BooleanLiteral(false) => {
                                SuperPropertyAccess::new(Sym::FALSE.into())
                            }
                            TokenKind::NullLiteral => SuperPropertyAccess::new(Sym::NULL.into()),
                            TokenKind::PrivateIdentifier(_) => {
                                return Err(Error::general(
                                    "unexpected private identifier",
                                    token.span().start(),
                                ));
                            }
                            _ => {
                                return Err(Error::unexpected(
                                    token.to_string(interner),
                                    token.span(),
                                    "expected super property",
                                ));
                            }
                        };
                        ast::Expression::PropertyAccess(field.into())
                    }
                    TokenKind::Punctuator(Punctuator::OpenBracket) => {
                        let expr = Expression::new(None, true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                        cursor.expect(Punctuator::CloseBracket, "super property", interner)?;
                        ast::Expression::PropertyAccess(
                            SuperPropertyAccess::new(expr.into()).into(),
                        )
                    }
                    _ => {
                        return Err(Error::unexpected(
                            token.to_string(interner),
                            token.span(),
                            "expected super property",
                        ))
                    }
                }
            }
            _ => PrimaryExpression::new(self.name, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?,
        };

        cursor.set_goal(InputElement::TemplateTail);

        while let Some(tok) = cursor.peek(0, interner)? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor
                        .next(interner)?
                        .expect("dot punctuator token disappeared"); // We move the parser forward.

                    let token = cursor.next(interner).or_abrupt()?;

                    let access = match token.kind() {
                        TokenKind::IdentifierName((name, _)) => {
                            SimplePropertyAccess::new(lhs, *name).into()
                        }
                        TokenKind::Keyword((kw, _)) => {
                            SimplePropertyAccess::new(lhs, kw.to_sym()).into()
                        }
                        TokenKind::BooleanLiteral(true) => {
                            SimplePropertyAccess::new(lhs, Sym::TRUE).into()
                        }
                        TokenKind::BooleanLiteral(false) => {
                            SimplePropertyAccess::new(lhs, Sym::FALSE).into()
                        }
                        TokenKind::NullLiteral => SimplePropertyAccess::new(lhs, Sym::NULL).into(),
                        TokenKind::PrivateIdentifier(name) => {
                            PrivatePropertyAccess::new(lhs, PrivateName::new(*name)).into()
                        }
                        _ => {
                            return Err(Error::expected(
                                ["identifier".to_owned()],
                                token.to_string(interner),
                                token.span(),
                                "member expression",
                            ));
                        }
                    };

                    lhs = ast::Expression::PropertyAccess(access);
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    cursor
                        .next(interner)?
                        .expect("open bracket punctuator token disappeared"); // We move the parser forward.
                    let idx = Expression::new(None, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(Punctuator::CloseBracket, "member expression", interner)?;
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
