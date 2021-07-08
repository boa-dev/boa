//! Update expression parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-update-expressions

use super::left_hand_side::LeftHandSideExpression;
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{node, op::UnaryOp, Node, Punctuator},
        lexer::TokenKind,
        parser::{
            expression::unary::UnaryExpression, AllowAwait, AllowYield, Cursor, DeclaredNames,
            ParseError, ParseResult, TokenParser,
        },
    },
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

    fn parse(self, cursor: &mut Cursor<R>, env: &mut DeclaredNames) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("UpdateExpression", "Parsing");

        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
        match tok.kind() {
            TokenKind::Punctuator(Punctuator::Inc) => {
                cursor.next()?.expect("Punctuator::Inc token disappeared");
                return Ok(node::UnaryOp::new(
                    UnaryOp::IncrementPre,
                    UnaryExpression::new(self.allow_yield, self.allow_await).parse(cursor, env)?,
                )
                .into());
            }
            TokenKind::Punctuator(Punctuator::Dec) => {
                cursor.next()?.expect("Punctuator::Dec token disappeared");
                return Ok(node::UnaryOp::new(
                    UnaryOp::DecrementPre,
                    UnaryExpression::new(self.allow_yield, self.allow_await).parse(cursor, env)?,
                )
                .into());
            }
            _ => {}
        }

        let lhs =
            LeftHandSideExpression::new(self.allow_yield, self.allow_await).parse(cursor, env)?;
        if let Some(tok) = cursor.peek(0)? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Inc) => {
                    cursor.next()?.expect("Punctuator::Inc token disappeared");
                    return Ok(node::UnaryOp::new(UnaryOp::IncrementPost, lhs).into());
                }
                TokenKind::Punctuator(Punctuator::Dec) => {
                    cursor.next()?.expect("Punctuator::Dec token disappeared");
                    return Ok(node::UnaryOp::new(UnaryOp::DecrementPost, lhs).into());
                }
                _ => {}
            }
        }

        Ok(lhs)
    }
}
