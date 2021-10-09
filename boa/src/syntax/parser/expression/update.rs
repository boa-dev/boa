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
            expression::unary::UnaryExpression, Cursor, ParseError, ParseResult, TokenParser,
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
pub(super) struct UpdateExpression<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for UpdateExpression<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("UpdateExpression", "Parsing");

        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
        match tok.kind() {
            TokenKind::Punctuator(Punctuator::Inc) => {
                cursor.next()?.expect("Punctuator::Inc token disappeared");
                return Ok(node::UnaryOp::new(
                    UnaryOp::IncrementPre,
                    UnaryExpression::<YIELD, AWAIT>.parse(cursor)?,
                )
                .into());
            }
            TokenKind::Punctuator(Punctuator::Dec) => {
                cursor.next()?.expect("Punctuator::Dec token disappeared");
                return Ok(node::UnaryOp::new(
                    UnaryOp::DecrementPre,
                    UnaryExpression::<YIELD, AWAIT>.parse(cursor)?,
                )
                .into());
            }
            _ => {}
        }

        let lhs = LeftHandSideExpression::<YIELD, AWAIT>.parse(cursor)?;
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
