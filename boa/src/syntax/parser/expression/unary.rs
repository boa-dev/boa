//! Unary operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
//! [spec]: https://tc39.es/ecma262/#sec-unary-operators

use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{
            node::{self, Node},
            op::UnaryOp,
            Keyword, Punctuator,
        },
        lexer::TokenKind,
        parser::{
            expression::update::UpdateExpression, AllowAwait, AllowYield, Cursor, ParseError,
            ParseResult, TokenParser,
        },
    },
};
use std::io::Read;

/// Parses a unary expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
/// [spec]: https://tc39.es/ecma262/#prod-UnaryExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct UnaryExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UnaryExpression {
    /// Creates a new `UnaryExpression` parser.
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

impl<R> TokenParser<R> for UnaryExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("UnaryExpression", "Parsing");

        let tok = cursor.peek(false)?.ok_or(ParseError::AbruptEnd)?;
        match tok.kind() {
            TokenKind::Keyword(Keyword::Delete) => {
                cursor.next(false)?.expect("Delete keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Delete, self.parse(cursor)?).into())
            }
            TokenKind::Keyword(Keyword::Void) => {
                cursor.next(false)?.expect("Void keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Void, self.parse(cursor)?).into())
            }
            TokenKind::Keyword(Keyword::TypeOf) => {
                cursor.next(false)?.expect("TypeOf keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::TypeOf, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Add) => {
                cursor.next(false)?.expect("+ token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Plus, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Sub) => {
                cursor.next(false)?.expect("- token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Minus, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Neg) => {
                cursor.next(false)?.expect("~ token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Tilde, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Not) => {
                cursor.next(false)?.expect("! token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Not, self.parse(cursor)?).into())
            }
            _ => UpdateExpression::new(self.allow_yield, self.allow_await).parse(cursor),
        }
    }
}
