//! Update expression parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-update-expressions

use super::left_hand_side::LeftHandSideExpression;
use crate::syntax::{
    ast::{node, op::UnaryOp, Node, Punctuator},
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::unary::UnaryExpression, AllowAwait, AllowYield, Cursor, ParseError,
        ParseResult, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Parses an update expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-UpdateExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct UpdateExpression {
    name: Option<Sym>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UpdateExpression {
    /// Creates a new `UpdateExpression` parser.
    pub(super) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Sym>>,
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

impl<R> TokenParser<R> for UpdateExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("UpdateExpression", "Parsing");

        let tok = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        match tok.kind() {
            TokenKind::Punctuator(Punctuator::Inc) => {
                cursor
                    .next(interner)?
                    .expect("Punctuator::Inc token disappeared");
                return Ok(node::UnaryOp::new(
                    UnaryOp::IncrementPre,
                    UnaryExpression::new(self.name, self.allow_yield, self.allow_await)
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
                    UnaryExpression::new(self.name, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?,
                )
                .into());
            }
            _ => {}
        }

        let position = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .span()
            .start();
        let lhs = LeftHandSideExpression::new(self.name, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        if cursor.strict_mode() {
            if let Node::Identifier(ident) = lhs {
                if ident.sym() == Sym::ARGUMENTS {
                    return Err(ParseError::lex(LexError::Syntax(
                        "unexpected identifier 'arguments' in strict mode".into(),
                        position,
                    )));
                } else if ident.sym() == Sym::EVAL {
                    return Err(ParseError::lex(LexError::Syntax(
                        "unexpected identifier 'eval' in strict mode".into(),
                        position,
                    )));
                }
            }
        }

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
                        Node::GetConstField(_) | Node::GetField(_) => true,
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
                        Node::GetConstField(_) | Node::GetField(_) => true,
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
