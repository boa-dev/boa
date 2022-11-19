//! Primary expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Primary_expressions
//! [spec]: https://tc39.es/ecma262/#prod-PrimaryExpression

#[cfg(test)]
mod tests;

mod array_initializer;
mod async_function_expression;
mod async_generator_expression;
mod class_expression;
mod function_expression;
mod generator_expression;
mod template;

pub(in crate::parser) mod object_initializer;

use self::{
    array_initializer::ArrayLiteral, async_function_expression::AsyncFunctionExpression,
    async_generator_expression::AsyncGeneratorExpression, class_expression::ClassExpression,
    function_expression::FunctionExpression, generator_expression::GeneratorExpression,
    object_initializer::ObjectLiteral,
};
use crate::{
    lexer::{token::Numeric, InputElement, Token, TokenKind},
    parser::{
        expression::{
            identifiers::IdentifierReference, primary::template::TemplateLiteral,
            BindingIdentifier, Expression,
        },
        statement::{ArrayBindingPattern, ObjectBindingPattern},
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    self as ast,
    declaration::Variable,
    expression::{
        literal::Literal,
        operator::{assign::AssignTarget, binary::BinaryOp},
        Call, Identifier, New,
    },
    function::{FormalParameter, FormalParameterList},
    operations::{contains, ContainsSymbol},
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    Keyword, Punctuator, Span,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

pub(in crate::parser) use object_initializer::Initializer;

/// Parses a primary expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Primary_expressions
/// [spec]: https://tc39.es/ecma262/#prod-PrimaryExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct PrimaryExpression {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PrimaryExpression {
    /// Creates a new `PrimaryExpression` parser.
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

impl<R> TokenParser<R> for PrimaryExpression
where
    R: Read,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("PrimaryExpression", "Parsing");

        // TODO: tok currently consumes the token instead of peeking, so the token
        // isn't passed and consumed by parsers according to spec (EX: GeneratorExpression)
        let tok = cursor.peek(0, interner).or_abrupt()?;
        let tok_position = tok.span().start();

        match tok.kind() {
            TokenKind::Keyword((Keyword::This, true)) => Err(Error::general(
                "Keyword must not contain escaped characters",
                tok_position,
            )),
            TokenKind::Keyword((Keyword::This, false)) => {
                cursor.advance(interner);
                Ok(ast::Expression::This)
            }
            TokenKind::Keyword((Keyword::Function, _)) => {
                cursor.advance(interner);
                let next_token = cursor.peek(0, interner).or_abrupt()?;
                if next_token.kind() == &TokenKind::Punctuator(Punctuator::Mul) {
                    GeneratorExpression::new(self.name)
                        .parse(cursor, interner)
                        .map(Into::into)
                } else {
                    FunctionExpression::new(self.name)
                        .parse(cursor, interner)
                        .map(Into::into)
                }
            }
            TokenKind::Keyword((Keyword::Class, _)) => {
                cursor.advance(interner);
                ClassExpression::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
                    .map(Into::into)
            }
            TokenKind::Keyword((Keyword::Async, contain_escaped_char)) => {
                let contain_escaped_char = *contain_escaped_char;
                match cursor.peek(1, interner)?.map(Token::kind) {
                    Some(TokenKind::Keyword((Keyword::Function, _))) if contain_escaped_char => {
                        Err(Error::general(
                            "Keyword must not contain escaped characters",
                            tok_position,
                        ))
                    }
                    Some(TokenKind::Keyword((Keyword::Function, _))) => {
                        cursor.advance(interner);
                        match cursor.peek(1, interner)?.map(Token::kind) {
                            Some(TokenKind::Punctuator(Punctuator::Mul)) => {
                                AsyncGeneratorExpression::new(self.name)
                                    .parse(cursor, interner)
                                    .map(Into::into)
                            }
                            _ => AsyncFunctionExpression::new(self.name, self.allow_yield)
                                .parse(cursor, interner)
                                .map(Into::into),
                        }
                    }
                    _ => IdentifierReference::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)
                        .map(Into::into),
                }
            }
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                cursor.advance(interner);
                cursor.set_goal(InputElement::RegExp);
                let expr = CoverParenthesizedExpressionAndArrowParameterList::new(
                    self.name,
                    self.allow_yield,
                    self.allow_await,
                )
                .parse(cursor, interner)?;
                Ok(expr)
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                cursor.advance(interner);
                cursor.set_goal(InputElement::RegExp);
                ArrayLiteral::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
                    .map(Into::into)
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                cursor.advance(interner);
                cursor.set_goal(InputElement::RegExp);
                ObjectLiteral::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
                    .map(Into::into)
            }
            TokenKind::BooleanLiteral(boolean) => {
                let node = Literal::from(*boolean).into();
                cursor.advance(interner);
                Ok(node)
            }
            TokenKind::NullLiteral => {
                cursor.advance(interner);
                Ok(Literal::Null.into())
            }
            TokenKind::Identifier(_)
            | TokenKind::Keyword((
                Keyword::Let | Keyword::Yield | Keyword::Await | Keyword::Of,
                _,
            )) => IdentifierReference::new(self.allow_yield, self.allow_await)
                .parse(cursor, interner)
                .map(Into::into),
            TokenKind::StringLiteral(lit) => {
                let node = Literal::from(*lit).into();
                cursor.advance(interner);
                Ok(node)
            }
            TokenKind::TemplateNoSubstitution(template_string) => {
                let node = Literal::from(
                    template_string
                        .to_owned_cooked(interner)
                        .map_err(Error::lex)?,
                )
                .into();
                cursor.advance(interner);
                Ok(node)
            }
            TokenKind::NumericLiteral(Numeric::Integer(num)) => {
                let node = Literal::from(*num).into();
                cursor.advance(interner);
                Ok(node)
            }
            TokenKind::NumericLiteral(Numeric::Rational(num)) => {
                let node = Literal::from(*num).into();
                cursor.advance(interner);
                Ok(node)
            }
            TokenKind::NumericLiteral(Numeric::BigInt(num)) => {
                let node = Literal::from(num.clone()).into();
                cursor.advance(interner);
                Ok(node)
            }
            TokenKind::RegularExpressionLiteral(body, flags) => {
                let node = ast::Expression::from(New::from(Call::new(
                    Identifier::new(Sym::REGEXP).into(),
                    vec![Literal::from(*body).into(), Literal::from(*flags).into()].into(),
                )));
                cursor.advance(interner);
                Ok(node)
            }
            TokenKind::Punctuator(Punctuator::Div) => {
                let position = tok.span().start();
                cursor.advance(interner);
                let tok = cursor.lex_regex(position, interner)?;

                if let TokenKind::RegularExpressionLiteral(body, flags) = *tok.kind() {
                    Ok(ast::Expression::from(New::from(Call::new(
                        Identifier::new(Sym::REGEXP).into(),
                        vec![Literal::from(body).into(), Literal::from(flags).into()].into(),
                    ))))
                } else {
                    // A regex was expected and nothing else.
                    Err(Error::unexpected(
                        tok.to_string(interner),
                        tok.span(),
                        "regular expression literal",
                    ))
                }
            }
            TokenKind::TemplateMiddle(template_string) => {
                let parser = TemplateLiteral::new(
                    self.allow_yield,
                    self.allow_await,
                    tok.span().start(),
                    template_string
                        .to_owned_cooked(interner)
                        .map_err(Error::lex)?,
                );
                cursor.advance(interner);
                parser.parse(cursor, interner).map(Into::into)
            }
            _ => Err(Error::unexpected(
                tok.to_string(interner),
                tok.span(),
                "primary expression",
            )),
        }
    }
}

/// Parses a `CoverParenthesizedExpressionAndArrowParameterList` expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CoverParenthesizedExpressionAndArrowParameterList
#[derive(Debug, Clone, Copy)]
pub(super) struct CoverParenthesizedExpressionAndArrowParameterList {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl CoverParenthesizedExpressionAndArrowParameterList {
    /// Creates a new `CoverParenthesizedExpressionAndArrowParameterList` parser.
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

impl<R> TokenParser<R> for CoverParenthesizedExpressionAndArrowParameterList
where
    R: Read,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        #[derive(Debug)]
        enum InnerExpression {
            Expression(ast::Expression),
            SpreadObject(Vec<ObjectPatternElement>),
            SpreadArray(Vec<ArrayPatternElement>),
            SpreadBinding(Identifier),
        }

        let _timer = Profiler::global().start_event(
            "CoverParenthesizedExpressionAndArrowParameterList",
            "Parsing",
        );

        let start_span = cursor.peek(0, interner).or_abrupt()?.span();

        let mut expressions = Vec::new();
        let mut tailing_comma = None;

        let next = cursor.peek(0, interner).or_abrupt()?;
        let span = match next.kind() {
            TokenKind::Punctuator(Punctuator::CloseParen) => {
                let span = next.span();
                cursor.advance(interner);
                span
            }
            TokenKind::Punctuator(Punctuator::Spread) => {
                cursor.advance(interner);
                let next = cursor.peek(0, interner).or_abrupt()?;
                match next.kind() {
                    TokenKind::Punctuator(Punctuator::OpenBlock) => {
                        let bindings =
                            ObjectBindingPattern::new(self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?;
                        expressions.push(InnerExpression::SpreadObject(bindings));
                    }
                    TokenKind::Punctuator(Punctuator::OpenBracket) => {
                        let bindings = ArrayBindingPattern::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                        expressions.push(InnerExpression::SpreadArray(bindings));
                    }
                    _ => {
                        let binding = BindingIdentifier::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                        expressions.push(InnerExpression::SpreadBinding(binding));
                    }
                }

                cursor
                    .expect(
                        Punctuator::CloseParen,
                        "CoverParenthesizedExpressionAndArrowParameterList",
                        interner,
                    )?
                    .span()
            }
            _ => {
                let expression =
                    Expression::new(self.name, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                expressions.push(InnerExpression::Expression(expression));

                let next = cursor.peek(0, interner).or_abrupt()?;
                match next.kind() {
                    TokenKind::Punctuator(Punctuator::CloseParen) => {
                        let span = next.span();
                        cursor.advance(interner);
                        span
                    }
                    TokenKind::Punctuator(Punctuator::Comma) => {
                        cursor.advance(interner);
                        let next = cursor.peek(0, interner).or_abrupt()?;
                        match next.kind() {
                            TokenKind::Punctuator(Punctuator::CloseParen) => {
                                let span = next.span();
                                tailing_comma = Some(next.span());
                                cursor.advance(interner);
                                span
                            }
                            TokenKind::Punctuator(Punctuator::Spread) => {
                                cursor.advance(interner);
                                let next = cursor.peek(0, interner).or_abrupt()?;
                                match next.kind() {
                                    TokenKind::Punctuator(Punctuator::OpenBlock) => {
                                        let bindings = ObjectBindingPattern::new(
                                            self.allow_yield,
                                            self.allow_await,
                                        )
                                        .parse(cursor, interner)?;
                                        expressions.push(InnerExpression::SpreadObject(bindings));
                                    }
                                    TokenKind::Punctuator(Punctuator::OpenBracket) => {
                                        let bindings = ArrayBindingPattern::new(
                                            self.allow_yield,
                                            self.allow_await,
                                        )
                                        .parse(cursor, interner)?;
                                        expressions.push(InnerExpression::SpreadArray(bindings));
                                    }
                                    _ => {
                                        let binding = BindingIdentifier::new(
                                            self.allow_yield,
                                            self.allow_await,
                                        )
                                        .parse(cursor, interner)?;
                                        expressions.push(InnerExpression::SpreadBinding(binding));
                                    }
                                }

                                cursor
                                    .expect(
                                        Punctuator::CloseParen,
                                        "CoverParenthesizedExpressionAndArrowParameterList",
                                        interner,
                                    )?
                                    .span()
                            }
                            _ => {
                                return Err(Error::expected(
                                    vec![")".to_owned(), "...".to_owned()],
                                    next.kind().to_string(interner),
                                    next.span(),
                                    "CoverParenthesizedExpressionAndArrowParameterList",
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(Error::expected(
                            vec![")".to_owned(), ",".to_owned()],
                            next.kind().to_string(interner),
                            next.span(),
                            "CoverParenthesizedExpressionAndArrowParameterList",
                        ))
                    }
                }
            }
        };

        let is_arrow = if cursor.peek(0, interner)?.map(Token::kind)
            == Some(&TokenKind::Punctuator(Punctuator::Arrow))
        {
            !cursor.peek_is_line_terminator(0, interner).or_abrupt()?
        } else {
            false
        };

        // If the next token is not an arrow, we know that we must parse a parenthesized expression.
        if !is_arrow {
            if let Some(span) = tailing_comma {
                return Err(Error::unexpected(
                    Punctuator::Comma,
                    span,
                    "trailing comma in parenthesized expression",
                ));
            }
            if expressions.is_empty() {
                return Err(Error::unexpected(
                    Punctuator::CloseParen,
                    span,
                    "empty parenthesized expression",
                ));
            }
            if expressions.len() != 1 {
                return Err(Error::unexpected(
                    Punctuator::CloseParen,
                    span,
                    "multiple expressions in parenthesized expression",
                ));
            }
            if let InnerExpression::Expression(expression) = &expressions[0] {
                return Ok(expression.clone());
            }
            return Err(Error::unexpected(
                Punctuator::CloseParen,
                span,
                "parenthesized expression with spread expressions",
            ));
        }

        // We know that we must parse an arrow function.
        // We parse the expressions in to a parameter list.

        let mut parameters = Vec::new();

        for expression in expressions {
            match expression {
                InnerExpression::Expression(node) => {
                    expression_to_formal_parameters(
                        &node,
                        &mut parameters,
                        cursor.strict_mode(),
                        start_span,
                    )?;
                }
                InnerExpression::SpreadObject(bindings) => {
                    let declaration = Variable::from_pattern(bindings.into(), None);
                    let parameter = FormalParameter::new(declaration, true);
                    parameters.push(parameter);
                }
                InnerExpression::SpreadArray(bindings) => {
                    let declaration = Variable::from_pattern(bindings.into(), None);
                    let parameter = FormalParameter::new(declaration, true);
                    parameters.push(parameter);
                }
                InnerExpression::SpreadBinding(ident) => {
                    let declaration = Variable::from_identifier(ident, None);
                    let parameter = FormalParameter::new(declaration, true);
                    parameters.push(parameter);
                }
            }
        }

        let parameters = FormalParameterList::from(parameters);

        if let Some(span) = tailing_comma {
            if parameters.has_rest_parameter() {
                return Err(Error::general(
                    "rest parameter must be last formal parameter",
                    span.start(),
                ));
            }
        }

        if contains(&parameters, ContainsSymbol::YieldExpression) {
            return Err(Error::general(
                "yield expression is not allowed in formal parameter list of arrow function",
                start_span.start(),
            ));
        }

        Ok(ast::Expression::FormalParameterList(parameters))
    }
}

/// Convert an expression to a formal parameter and append it to the given parameter list.
fn expression_to_formal_parameters(
    node: &ast::Expression,
    parameters: &mut Vec<FormalParameter>,
    strict: bool,
    span: Span,
) -> ParseResult<()> {
    match node {
        ast::Expression::Identifier(identifier) if strict && *identifier == Sym::EVAL => {
            return Err(Error::general(
                "parameter name 'eval' not allowed in strict mode",
                span.start(),
            ));
        }
        ast::Expression::Identifier(identifier) if strict && *identifier == Sym::ARGUMENTS => {
            return Err(Error::general(
                "parameter name 'arguments' not allowed in strict mode",
                span.start(),
            ));
        }
        ast::Expression::Identifier(identifier) => {
            parameters.push(FormalParameter::new(
                Variable::from_identifier(*identifier, None),
                false,
            ));
        }
        ast::Expression::Binary(bin_op) if bin_op.op() == BinaryOp::Comma => {
            expression_to_formal_parameters(bin_op.lhs(), parameters, strict, span)?;
            expression_to_formal_parameters(bin_op.rhs(), parameters, strict, span)?;
        }
        ast::Expression::Assign(assign) => match assign.lhs() {
            AssignTarget::Identifier(ident) => {
                parameters.push(FormalParameter::new(
                    Variable::from_identifier(*ident, Some(assign.rhs().clone())),
                    false,
                ));
            }
            AssignTarget::Pattern(pattern) => match pattern {
                Pattern::Object(pattern) => {
                    parameters.push(FormalParameter::new(
                        Variable::from_pattern(
                            pattern.bindings().to_vec().into(),
                            Some(assign.rhs().clone()),
                        ),
                        false,
                    ));
                }
                Pattern::Array(pattern) => {
                    parameters.push(FormalParameter::new(
                        Variable::from_pattern(
                            pattern.bindings().to_vec().into(),
                            Some(assign.rhs().clone()),
                        ),
                        false,
                    ));
                }
            },
            AssignTarget::Access(_) => {
                return Err(Error::general(
                    "invalid initialization expression in formal parameter list",
                    span.start(),
                ));
            }
        },
        ast::Expression::ObjectLiteral(object) => {
            let pattern = object.to_pattern(strict).ok_or_else(|| {
                Error::general(
                    "invalid object binding pattern in formal parameter list",
                    span.start(),
                )
            })?;

            parameters.push(FormalParameter::new(
                Variable::from_pattern(pattern.into(), None),
                false,
            ));
        }
        ast::Expression::ArrayLiteral(array) => {
            let pattern = array.to_pattern(strict).ok_or_else(|| {
                Error::general(
                    "invalid array binding pattern in formal parameter list",
                    span.start(),
                )
            })?;

            parameters.push(FormalParameter::new(
                Variable::from_pattern(pattern.into(), None),
                false,
            ));
        }
        _ => {
            return Err(Error::unexpected(
                ")".to_string(),
                span,
                "parenthesized expression with non-binding expression",
            ));
        }
    }
    Ok(())
}
