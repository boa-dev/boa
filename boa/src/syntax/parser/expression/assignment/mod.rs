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

use self::{arrow_function::ArrowFunction, conditional::ConditionalExpression};
use crate::syntax::lexer::{Error as LexError, InputElement, TokenKind};
use crate::{
    syntax::{
        ast::{
            node::{Assign, BinOp, Node, NodeKind},
            Keyword, Punctuator, Span,
        },
        parser::{AllowAwait, AllowIn, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
    },
    BoaProfiler,
};
pub(super) use exponentiation::ExponentiationExpression;

use std::io::Read;

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
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AssignmentExpression {
    /// Creates a new `AssignmentExpression` parser.
    pub(in crate::syntax::parser) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
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

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("AssignmentExpression", "Parsing");
        cursor.set_goal(InputElement::Div);

        // Arrow function
        match cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind() {
            // a=>{}
            TokenKind::Identifier(_)
            | TokenKind::Keyword(Keyword::Yield)
            | TokenKind::Keyword(Keyword::Await) => {
                if let Ok(tok) = cursor.peek_expect_no_lineterminator(1, "assignment expression") {
                    if tok.kind() == &TokenKind::Punctuator(Punctuator::Arrow) {
                        let (decl, span) =
                            ArrowFunction::new(self.allow_in, self.allow_yield, self.allow_await)
                                .parse(cursor)?;

                        return Ok(Node::new(NodeKind::ArrowFunctionDecl(decl), span));
                    }
                }
            }

            // (a,b)=>{} or (a,b) or (Expression)
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                if let Some(next_token) = cursor.peek(1)? {
                    match *next_token.kind() {
                        TokenKind::Punctuator(Punctuator::CloseParen) => {
                            // Need to check if the token after the close paren is an arrow, if so then this is an ArrowFunction
                            // otherwise it is an expression of the form (b).
                            if let Some(t) = cursor.peek(2)? {
                                if t.kind() == &TokenKind::Punctuator(Punctuator::Arrow) {
                                    let (decl, span) = ArrowFunction::new(
                                        self.allow_in,
                                        self.allow_yield,
                                        self.allow_await,
                                    )
                                    .parse(cursor)?;

                                    return Ok(Node::new(NodeKind::ArrowFunctionDecl(decl), span));
                                }
                            }
                        }
                        TokenKind::Punctuator(Punctuator::Spread) => {
                            let (decl, span) = ArrowFunction::new(
                                self.allow_in,
                                self.allow_yield,
                                self.allow_await,
                            )
                            .parse(cursor)?;

                            return Ok(Node::new(NodeKind::ArrowFunctionDecl(decl), span));
                        }
                        TokenKind::Identifier(_) => {
                            if let Some(t) = cursor.peek(2)? {
                                match *t.kind() {
                                    TokenKind::Punctuator(Punctuator::Comma) => {
                                        // This must be an argument list and therefore (a, b) => {}
                                        let (decl, span) = ArrowFunction::new(
                                            self.allow_in,
                                            self.allow_yield,
                                            self.allow_await,
                                        )
                                        .parse(cursor)?;

                                        return Ok(Node::new(
                                            NodeKind::ArrowFunctionDecl(decl),
                                            span,
                                        ));
                                    }
                                    TokenKind::Punctuator(Punctuator::CloseParen) => {
                                        // Need to check if the token after the close paren is an arrow, if so then this is an ArrowFunction
                                        // otherwise it is an expression of the form (b).
                                        if let Some(t) = cursor.peek(3)? {
                                            if t.kind() == &TokenKind::Punctuator(Punctuator::Arrow)
                                            {
                                                let (decl, span) = ArrowFunction::new(
                                                    self.allow_in,
                                                    self.allow_yield,
                                                    self.allow_await,
                                                )
                                                .parse(cursor)?;

                                                return Ok(Node::new(
                                                    NodeKind::ArrowFunctionDecl(decl),
                                                    span,
                                                ));
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

        let mut lhs = ConditionalExpression::new(self.allow_in, self.allow_yield, self.allow_await)
            .parse(cursor)?;

        // Review if we are trying to assign to an invalid left hand side expression.
        // TODO: can we avoid cloning?
        if let Some(tok) = cursor.peek(0)?.cloned() {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Assign) => {
                    cursor.next()?.expect("= token vanished"); // Consume the token.
                    if lhs.kind().is_assignable() {
                        let expr = self.parse(cursor)?;
                        let span = Span::new(lhs.span().start(), expr.span().end());

                        lhs = Node::new(Assign::new(lhs, expr), span);
                    } else {
                        return Err(ParseError::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            tok.span().start(),
                        )));
                    }
                }
                TokenKind::Punctuator(p) if p.as_binop().is_some() && p != &Punctuator::Comma => {
                    cursor.next()?.expect("token vanished"); // Consume the token.
                    if lhs.kind().is_assignable() {
                        let binop = p.as_binop().expect("binop disappeared");
                        let expr = self.parse(cursor)?;

                        let span = Span::new(lhs.span().start(), expr.span().end());

                        lhs = Node::new(BinOp::new(binop, lhs, expr), span);
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
