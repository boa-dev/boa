//! Update expression parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-update-expressions

use boa_interner::Sym;

use super::left_hand_side::LeftHandSideExpression;
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{node, op::UnaryOp, Node, Punctuator},
        lexer::{Error as LexError, TokenKind},
        parser::{
            expression::unary::UnaryExpression, AllowAwait, AllowYield, Cursor, ParseError,
            ParseResult, TokenParser,
        },
    },
    Interner,
};

use std::io::Read;

/// Parses an update expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-UpdateExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct UpdateExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UpdateExpression {
    /// Creates a new `UpdateExpression` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for UpdateExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("UpdateExpression", "Parsing");

        let tok = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        match tok.kind() {
            TokenKind::Punctuator(Punctuator::Inc) => {
                cursor
                    .next(interner)?
                    .expect("Punctuator::Inc token disappeared");
                return Ok(node::UnaryOp::new(
                    UnaryOp::IncrementPre,
                    UnaryExpression::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?,
                )
                .into());
            }
            TokenKind::Punctuator(Punctuator::Dec) => {
                cursor
                    .next(interner)?
                    .expect("Punctuator::Dec token disappeared");
                return Ok(node::UnaryOp::new(
                    UnaryOp::DecrementPre,
                    UnaryExpression::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?,
                )
                .into());
            }
            _ => {}
        }

        let lhs = LeftHandSideExpression::new(self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;
        let strict = cursor.strict_mode();
        if let Some(tok) = cursor.peek(0, interner)? {
            let token_start = tok.span().start();
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Inc) => {
                    cursor
                        .next(interner)?
                        .expect("Punctuator::Inc token disappeared");
                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    let ok = match &lhs {
                        Node::Identifier(_) if !strict => true,
                        Node::Identifier(ident)
                            if ![Sym::EVAL, Sym::ARGUMENTS].contains(&ident.sym()) =>
                        {
                            true
                        }
                        Node::GetConstField(_) => true,
                        Node::GetField(_) => true,
                        _ => false,
                    };
                    if !ok {
                        return Err(ParseError::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            token_start,
                        )));
                    }

                    return Ok(node::UnaryOp::new(UnaryOp::IncrementPost, lhs).into());
                }
                TokenKind::Punctuator(Punctuator::Dec) => {
                    cursor
                        .next(interner)?
                        .expect("Punctuator::Dec token disappeared");
                    // https://tc39.es/ecma262/#sec-update-expressions-static-semantics-early-errors
                    let ok = match &lhs {
                        Node::Identifier(_) if !strict => true,
                        Node::Identifier(ident)
                            if ![Sym::EVAL, Sym::ARGUMENTS].contains(&ident.sym()) =>
                        {
                            true
                        }
                        Node::GetConstField(_) => true,
                        Node::GetField(_) => true,
                        _ => false,
                    };
                    if !ok {
                        return Err(ParseError::lex(LexError::Syntax(
                            "Invalid left-hand side in assignment".into(),
                            token_start,
                        )));
                    }

                    return Ok(node::UnaryOp::new(UnaryOp::DecrementPost, lhs).into());
                }
                _ => {}
            }
        }

        Ok(lhs)
    }
}
