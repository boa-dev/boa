//! Assignment operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Assignment
//! [spec]: https://tc39.es/ecma262/#sec-assignment-operators

mod arrow_function;
mod async_arrow_function;
mod conditional;
mod exponentiation;
mod r#yield;

use crate::{
    lexer::{Error as LexError, InputElement, TokenKind},
    parser::{
        expression::assignment::{
            arrow_function::{ArrowFunction, ConciseBody},
            async_arrow_function::AsyncArrowFunction,
            conditional::ConditionalExpression,
            r#yield::YieldExpression,
        },
        name_in_lexically_declared_names, AllowAwait, AllowIn, AllowYield, Cursor, OrAbrupt,
        ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    expression::{
        operator::assign::{Assign, AssignOp, AssignTarget},
        Identifier,
    },
    operations::{bound_names, contains, top_level_lexically_declared_names, ContainsSymbol},
    Expression, Keyword, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

pub(super) use exponentiation::ExponentiationExpression;

use super::check_strict_arguments_or_eval;

/// Assignment expression parsing.
///
/// This can be one of the following:
///
///  - [`ConditionalExpression`](../conditional_operator/struct.ConditionalExpression.html)
///  - `YieldExpression`
///  - [`ArrowFunction`](../../function/arrow_function/struct.ArrowFunction.html)
///  - `AsyncArrowFunction`
///  - [`LeftHandSideExpression`][lhs] `=` `AssignmentExpression`
///  - [`LeftHandSideExpression`][lhs] `AssignmentOperator` `AssignmentExpression`
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Assignment
/// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
/// [lhs]: ../lhs_expression/struct.LeftHandSideExpression.html
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct AssignmentExpression {
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AssignmentExpression {
    /// Creates a new `AssignmentExpression` parser.
    pub(in crate::parser) fn new<N, I, Y, A>(
        name: N,
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for AssignmentExpression
where
    R: Read,
{
    type Output = Expression;

    fn parse(mut self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Expression> {
        let _timer = Profiler::global().start_event("AssignmentExpression", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        match cursor.peek(0, interner).or_abrupt()?.kind() {
            // [+Yield]YieldExpression[?In, ?Await]
            TokenKind::Keyword((Keyword::Yield, _)) if self.allow_yield.0 => {
                return YieldExpression::new(self.allow_in, self.allow_await)
                    .parse(cursor, interner)
            }
            // ArrowFunction[?In, ?Yield, ?Await] -> ArrowParameters[?Yield, ?Await] -> BindingIdentifier[?Yield, ?Await]
            TokenKind::Identifier(_) | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                cursor.set_goal(InputElement::Div);

                // Because we already peeked the identifier token, there may be a line terminator before the identifier token.
                // In that case we have to skip an additional token on the next peek.
                let skip_n = if cursor.peek_is_line_terminator(0, interner).or_abrupt()? {
                    2
                } else {
                    1
                };
                if let Ok(tok) =
                    cursor.peek_expect_no_lineterminator(skip_n, "assignment expression", interner)
                {
                    if tok.kind() == &TokenKind::Punctuator(Punctuator::Arrow) {
                        return ArrowFunction::new(
                            self.name,
                            self.allow_in,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)
                        .map(Expression::ArrowFunction);
                    }
                }
            }
            //  AsyncArrowFunction[?In, ?Yield, ?Await]
            TokenKind::Keyword((Keyword::Async, _)) => {
                let skip_n = if cursor.peek_is_line_terminator(0, interner).or_abrupt()? {
                    2
                } else {
                    1
                };

                if !cursor
                    .peek_is_line_terminator(skip_n, interner)
                    .or_abrupt()?
                    && matches!(
                        cursor.peek(1, interner).or_abrupt()?.kind(),
                        TokenKind::Identifier(_)
                            | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _))
                            | TokenKind::Punctuator(Punctuator::OpenParen)
                    )
                {
                    return Ok(
                        AsyncArrowFunction::new(self.name, self.allow_in, self.allow_yield)
                            .parse(cursor, interner)?
                            .into(),
                    );
                }
            }
            _ => {}
        }

        cursor.set_goal(InputElement::Div);

        let position = cursor.peek(0, interner).or_abrupt()?.span().start();
        let mut lhs = ConditionalExpression::new(
            self.name,
            self.allow_in,
            self.allow_yield,
            self.allow_await,
        )
        .parse(cursor, interner)?;

        // If the left hand side is a parameter list, we must parse an arrow function.
        if let Expression::FormalParameterList(parameters) = lhs {
            cursor.peek_expect_no_lineterminator(0, "arrow function", interner)?;

            cursor.expect(
                TokenKind::Punctuator(Punctuator::Arrow),
                "arrow function",
                interner,
            )?;
            let arrow = cursor.arrow();
            cursor.set_arrow(true);
            let body = ConciseBody::new(self.allow_in).parse(cursor, interner)?;
            cursor.set_arrow(arrow);

            // Early Error: ArrowFormalParameters are UniqueFormalParameters.
            if parameters.has_duplicates() {
                return Err(Error::lex(LexError::Syntax(
                    "Duplicate parameter name not allowed in this context".into(),
                    position,
                )));
            }

            // Early Error: It is a Syntax Error if ArrowParameters Contains YieldExpression is true.
            if contains(&parameters, ContainsSymbol::YieldExpression) {
                return Err(Error::lex(LexError::Syntax(
                    "Yield expression not allowed in this context".into(),
                    position,
                )));
            }

            // Early Error: It is a Syntax Error if ArrowParameters Contains AwaitExpression is true.
            if contains(&parameters, ContainsSymbol::AwaitExpression) {
                return Err(Error::lex(LexError::Syntax(
                    "Await expression not allowed in this context".into(),
                    position,
                )));
            }

            // Early Error: It is a Syntax Error if ConciseBodyContainsUseStrict of ConciseBody is true
            // and IsSimpleParameterList of ArrowParameters is false.
            if body.strict() && !parameters.is_simple() {
                return Err(Error::lex(LexError::Syntax(
                    "Illegal 'use strict' directive in function with non-simple parameter list"
                        .into(),
                    position,
                )));
            }

            // It is a Syntax Error if any element of the BoundNames of ArrowParameters
            // also occurs in the LexicallyDeclaredNames of ConciseBody.
            // https://tc39.es/ecma262/#sec-arrow-function-definitions-static-semantics-early-errors
            name_in_lexically_declared_names(
                &bound_names(&parameters),
                &top_level_lexically_declared_names(&body),
                position,
            )?;

            return Ok(boa_ast::function::ArrowFunction::new(self.name, parameters, body).into());
        }

        // Review if we are trying to assign to an invalid left hand side expression.
        if let Some(tok) = cursor.peek(0, interner)?.cloned() {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Assign) => {
                    if cursor.strict_mode() {
                        if let Expression::Identifier(ident) = lhs {
                            check_strict_arguments_or_eval(ident, position)?;
                        }
                    }

                    cursor.advance(interner);
                    cursor.set_goal(InputElement::RegExp);

                    if let Some(target) =
                        AssignTarget::from_expression(&lhs, cursor.strict_mode(), true)
                    {
                        if let AssignTarget::Identifier(ident) = target {
                            self.name = Some(ident);
                        }
                        let expr = self.parse(cursor, interner)?;
                        lhs = Assign::new(AssignOp::Assign, target, expr).into();
                    } else {
                        return Err(Error::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            tok.span().start(),
                        )));
                    }
                }
                TokenKind::Punctuator(p) if p.as_assign_op().is_some() => {
                    if cursor.strict_mode() {
                        if let Expression::Identifier(ident) = lhs {
                            check_strict_arguments_or_eval(ident, position)?;
                        }
                    }

                    cursor.advance(interner);
                    if let Some(target) =
                        AssignTarget::from_expression(&lhs, cursor.strict_mode(), false)
                    {
                        let assignop = p.as_assign_op().expect("assignop disappeared");
                        if assignop == AssignOp::BoolAnd
                            || assignop == AssignOp::BoolOr
                            || assignop == AssignOp::Coalesce
                        {
                            if let AssignTarget::Identifier(ident) = target {
                                self.name = Some(ident);
                            }
                        }

                        let rhs = self.parse(cursor, interner)?;
                        lhs = Assign::new(assignop, target, rhs).into();
                    } else {
                        return Err(Error::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            tok.span().start(),
                        )));
                    }
                }
                _ => {}
            }
        }

        Ok(lhs)
    }
}
