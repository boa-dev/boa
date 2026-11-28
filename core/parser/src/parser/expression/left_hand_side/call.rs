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
    Error,
    lexer::{Token, TokenKind},
    parser::{
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
        expression::{
            Expression, expression_to_formal_parameters,
            fpl_or_exp::FormalParameterListOrExpression,
            left_hand_side::template::TaggedTemplateLiteral,
        },
    },
    source::ReadChar,
};
use ast::function::PrivateName;
use boa_ast::{
    self as ast, Punctuator, Span, Spanned,
    declaration::Variable,
    expression::{
        Call, Identifier,
        access::{PrivatePropertyAccess, SimplePropertyAccess},
    },
    function::{FormalParameter, FormalParameterList},
    operations::bound_names,
};
use boa_interner::{Interner, Sym};

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
    R: ReadChar,
{
    type Output = FormalParameterListOrExpression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let token = cursor.peek(0, interner).or_abrupt()?;

        if token.kind() != &TokenKind::Punctuator(Punctuator::OpenParen) {
            let next_token = cursor.next(interner)?.expect("token vanished");
            return Err(Error::expected(
                ["(".to_owned()],
                next_token.to_string(interner),
                next_token.span(),
                "call expression",
            ));
        }

        let has_lt = cursor.peek_is_line_terminator(0, interner)?.unwrap_or(true);
        let (args, args_span, has_trailing_comma) =
            Arguments::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let is_async = match &self.first_member_expr {
            ast::Expression::Identifier(ident) => {
                ident.sym() == Sym::ASYNC
                    && (ident
                        .span()
                        .end()
                        .column_number()
                        .saturating_sub(ident.span().start().column_number())
                        == 5)
            }
            _ => false,
        };

        let is_arrow = is_async
            && !has_lt
            && cursor.peek(0, interner)?.map(Token::kind)
                == Some(&TokenKind::Punctuator(Punctuator::Arrow))
            && !cursor.peek_is_line_terminator(0, interner)?.unwrap_or(true);

        if is_arrow {
            let fpl = arguments_to_formal_parameters(
                &args,
                has_trailing_comma,
                cursor.strict(),
                args_span,
            )?;
            return Ok(FormalParameterListOrExpression::AsyncArrowHead {
                fpl,
                params_start_position: args_span.start(),
            });
        }

        let lhs = Call::new(self.first_member_expr, args, args_span).into();
        let expr = CallExpressionTail::new(self.allow_yield, self.allow_await, lhs)
            .parse(cursor, interner)?;
        Ok(FormalParameterListOrExpression::Expression(expr))
    }
}

/// Parses the tail parts of a call expression (property access, successive call, array access).
#[derive(Debug)]
pub(super) struct CallExpressionTail {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    call: ast::Expression,
}

impl CallExpressionTail {
    /// Creates a new `CallExpressionTail` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, call: ast::Expression) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            call,
        }
    }
}

impl<R> TokenParser<R> for CallExpressionTail
where
    R: ReadChar,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let mut lhs = self.call;

        while let Some(token) = cursor.peek(0, interner)?.cloned() {
            let lhs_span_start = lhs.span().start();
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let (args, args_span, _) = Arguments::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    lhs = Call::new(lhs, args, args_span).into();
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor.advance(interner);

                    let token = cursor.next(interner).or_abrupt()?;
                    let access = match token.kind() {
                        TokenKind::IdentifierName((name, _)) => {
                            SimplePropertyAccess::new(lhs, Identifier::new(*name, token.span()))
                                .into()
                        }
                        TokenKind::Keyword((kw, _)) => SimplePropertyAccess::new(
                            lhs,
                            Identifier::new(kw.to_sym(), token.span()),
                        )
                        .into(),
                        TokenKind::BooleanLiteral((true, _)) => {
                            SimplePropertyAccess::new(lhs, Identifier::new(Sym::TRUE, token.span()))
                                .into()
                        }
                        TokenKind::BooleanLiteral((false, _)) => SimplePropertyAccess::new(
                            lhs,
                            Identifier::new(Sym::FALSE, token.span()),
                        )
                        .into(),
                        TokenKind::NullLiteral(_) => {
                            SimplePropertyAccess::new(lhs, Identifier::new(Sym::NULL, token.span()))
                                .into()
                        }
                        TokenKind::PrivateIdentifier(name) => PrivatePropertyAccess::new(
                            lhs,
                            PrivateName::new(*name, token.span()),
                            Span::new(lhs_span_start, token.span().end()),
                        )
                        .into(),
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
                    let idx = Expression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(Punctuator::CloseBracket, "call expression", interner)?;
                    lhs =
                        ast::Expression::PropertyAccess(SimplePropertyAccess::new(lhs, idx).into());
                }
                TokenKind::TemplateNoSubstitution { .. } | TokenKind::TemplateMiddle { .. } => {
                    lhs = TaggedTemplateLiteral::new(
                        self.allow_yield,
                        self.allow_await,
                        token.start_group(),
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

/// Convert call arguments to a formal parameter list for an async arrow function.
fn arguments_to_formal_parameters(
    args: &[ast::Expression],
    has_trailing_comma: bool,
    strict: bool,
    args_span: Span,
) -> ParseResult<FormalParameterList> {
    let mut parameters = Vec::new();
    let num_args = args.len();

    for (i, arg) in args.iter().enumerate() {
        match arg {
            ast::Expression::Spread(spread) => {
                if i != num_args - 1 {
                    return Err(Error::general(
                        "rest parameter must be last formal parameter",
                        spread.span().start(),
                    ));
                }
                if has_trailing_comma {
                    return Err(Error::general(
                        "rest parameter must be last formal parameter",
                        args_span.end(),
                    ));
                }
                match spread.target() {
                    ast::Expression::Identifier(ident) => {
                        if strict && (*ident == Sym::EVAL || *ident == Sym::ARGUMENTS) {
                            return Err(Error::general(
                                format!(
                                    "parameter name '{}' not allowed in strict mode",
                                    if *ident == Sym::EVAL {
                                        "eval"
                                    } else {
                                        "arguments"
                                    }
                                ),
                                spread.span().start(),
                            ));
                        }
                        let declaration = Variable::from_identifier(*ident, None);
                        parameters.push(FormalParameter::new(declaration, true));
                    }
                    ast::Expression::ObjectLiteral(object) => {
                        let pattern = object.to_pattern(strict).ok_or_else(|| {
                            Error::general(
                                "invalid object binding pattern in formal parameter list",
                                spread.span().start(),
                            )
                        })?;
                        let declaration = Variable::from_pattern(pattern.into(), None);
                        parameters.push(FormalParameter::new(declaration, true));
                    }
                    ast::Expression::ArrayLiteral(array) => {
                        let pattern = array.to_pattern(strict).ok_or_else(|| {
                            Error::general(
                                "invalid array binding pattern in formal parameter list",
                                spread.span().start(),
                            )
                        })?;
                        let declaration = Variable::from_pattern(pattern.into(), None);
                        parameters.push(FormalParameter::new(declaration, true));
                    }
                    _ => {
                        return Err(Error::unexpected(
                            ")".to_string(),
                            spread.span(),
                            "parenthesized expression with non-binding expression",
                        ));
                    }
                }
            }
            expr => {
                expression_to_formal_parameters(expr, &mut parameters, strict, args_span)?;
            }
        }
    }

    let parameters = FormalParameterList::from(parameters);

    if bound_names(&parameters).contains(&Sym::AWAIT) {
        return Err(Error::general(
            "keyword `await` not allowed in this context",
            args_span.start(),
        ));
    }

    Ok(parameters)
}
