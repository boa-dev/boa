//! Assignment operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators#Assignment
//! [spec]: https://tc39.es/ecma262/#sec-assignment-operators

mod arrow_function;
mod conditional;
mod exponentiation;
mod r#yield;

use crate::syntax::{
    ast::{
        node::{operator::assign::AssignTarget, Assign, BinOp, Node},
        Keyword, Punctuator,
    },
    lexer::{Error as LexError, InputElement, TokenKind},
    parser::{
        expression::assignment::{
            arrow_function::ArrowFunction, conditional::ConditionalExpression,
            r#yield::YieldExpression,
        },
        AllowAwait, AllowIn, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

pub(super) use exponentiation::ExponentiationExpression;

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
pub(in crate::syntax::parser) struct AssignmentExpression {
    name: Option<Sym>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AssignmentExpression {
    /// Creates a new `AssignmentExpression` parser.
    pub(in crate::syntax::parser) fn new<N, I, Y, A>(
        name: N,
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        N: Into<Option<Sym>>,
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
    type Output = Node;

    fn parse(mut self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("AssignmentExpression", "Parsing");
        cursor.set_goal(InputElement::Div);

        match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            // [+Yield]YieldExpression[?In, ?Await]
            TokenKind::Keyword((Keyword::Yield, _)) if self.allow_yield.0 => {
                return YieldExpression::new(self.allow_in, self.allow_await)
                    .parse(cursor, interner)
            }
            // ArrowFunction[?In, ?Yield, ?Await] -> ArrowParameters[?Yield, ?Await] -> BindingIdentifier[?Yield, ?Await]
            TokenKind::Identifier(_) | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                if let Ok(tok) =
                    cursor.peek_expect_no_lineterminator(1, "assignment expression", interner)
                {
                    if tok.kind() == &TokenKind::Punctuator(Punctuator::Arrow) {
                        return ArrowFunction::new(
                            self.name,
                            self.allow_in,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)
                        .map(Node::ArrowFunctionDecl);
                    }
                }
            }
            // ArrowFunction[?In, ?Yield, ?Await] -> ArrowParameters[?Yield, ?Await] -> CoverParenthesizedExpressionAndArrowParameterList[?Yield, ?Await]
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                if let Some(next_token) = cursor.peek(1, interner)? {
                    match *next_token.kind() {
                        TokenKind::Punctuator(Punctuator::CloseParen) => {
                            // Need to check if the token after the close paren is an arrow, if so then this is an ArrowFunction
                            // otherwise it is an expression of the form (b).
                            if let Some(t) = cursor.peek(2, interner)? {
                                if t.kind() == &TokenKind::Punctuator(Punctuator::Arrow) {
                                    return ArrowFunction::new(
                                        self.name,
                                        self.allow_in,
                                        self.allow_yield,
                                        self.allow_await,
                                    )
                                    .parse(cursor, interner)
                                    .map(Node::ArrowFunctionDecl);
                                }
                            }
                        }
                        TokenKind::Punctuator(Punctuator::Spread) => {
                            return ArrowFunction::new(
                                None,
                                self.allow_in,
                                self.allow_yield,
                                self.allow_await,
                            )
                            .parse(cursor, interner)
                            .map(Node::ArrowFunctionDecl);
                        }
                        TokenKind::Identifier(_) => {
                            if let Some(t) = cursor.peek(2, interner)? {
                                match *t.kind() {
                                    TokenKind::Punctuator(Punctuator::Comma) => {
                                        // This must be an argument list and therefore (a, b) => {}
                                        return ArrowFunction::new(
                                            self.name,
                                            self.allow_in,
                                            self.allow_yield,
                                            self.allow_await,
                                        )
                                        .parse(cursor, interner)
                                        .map(Node::ArrowFunctionDecl);
                                    }
                                    TokenKind::Punctuator(Punctuator::CloseParen) => {
                                        // Need to check if the token after the close paren is an
                                        // arrow, if so then this is an ArrowFunction otherwise it
                                        // is an expression of the form (b).
                                        if let Some(t) = cursor.peek(3, interner)? {
                                            if t.kind() == &TokenKind::Punctuator(Punctuator::Arrow)
                                            {
                                                return ArrowFunction::new(
                                                    self.name,
                                                    self.allow_in,
                                                    self.allow_yield,
                                                    self.allow_await,
                                                )
                                                .parse(cursor, interner)
                                                .map(Node::ArrowFunctionDecl);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }

        cursor.set_goal(InputElement::Div);

        let position = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .span()
            .start();
        let mut lhs = ConditionalExpression::new(
            self.name,
            self.allow_in,
            self.allow_yield,
            self.allow_await,
        )
        .parse(cursor, interner)?;

        // Review if we are trying to assign to an invalid left hand side expression.
        // TODO: can we avoid cloning?
        if let Some(tok) = cursor.peek(0, interner)?.cloned() {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Assign) => {
                    if cursor.strict_mode() {
                        if let Node::Identifier(ident) = lhs {
                            ident.check_strict_arguments_or_eval(position)?;
                        }
                    }

                    cursor.next(interner)?.expect("= token vanished");
                    if let Some(target) = AssignTarget::from_node(&lhs, cursor.strict_mode()) {
                        if let AssignTarget::Identifier(ident) = target {
                            self.name = Some(ident.sym());
                        }
                        let expr = self.parse(cursor, interner)?;
                        lhs = Assign::new(target, expr).into();
                    } else {
                        return Err(ParseError::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            tok.span().start(),
                        )));
                    }
                }
                TokenKind::Punctuator(p) if p.as_binop().is_some() && p != &Punctuator::Comma => {
                    if cursor.strict_mode() {
                        if let Node::Identifier(ident) = lhs {
                            ident.check_strict_arguments_or_eval(position)?;
                        }
                    }

                    cursor.next(interner)?.expect("token vanished");
                    if is_assignable(&lhs) {
                        let binop = p.as_binop().expect("binop disappeared");
                        let expr = self.parse(cursor, interner)?;
                        lhs = BinOp::new(binop, lhs, expr).into();
                    } else {
                        return Err(ParseError::lex(LexError::Syntax(
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

/// Returns true if as per spec[spec] the node can be assigned a value.
///
/// [spec]: https://tc39.es/ecma262/#sec-assignment-operators-static-semantics-early-errors
#[inline]
pub(crate) fn is_assignable(node: &Node) -> bool {
    matches!(
        node,
        Node::GetConstField(_)
            | Node::GetField(_)
            | Node::Assign(_)
            | Node::Call(_)
            | Node::Identifier(_)
            | Node::Object(_)
    )
}
