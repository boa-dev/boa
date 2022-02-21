//! Unary operator parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Unary
//! [spec]: https://tc39.es/ecma262/#sec-unary-operators

use crate::syntax::{
    ast::{
        node::{self, Node},
        op::UnaryOp,
        Keyword, Punctuator,
    },
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::update::UpdateExpression, AllowAwait, AllowYield, Cursor, ParseError,
        ParseResult, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
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
pub(in crate::syntax::parser) struct UnaryExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UnaryExpression {
    /// Creates a new `UnaryExpression` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("UnaryExpression", "Parsing");

        let tok = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let token_start = tok.span().start();
        match tok.kind() {
            TokenKind::Keyword(Keyword::Delete) => {
                cursor.next(interner)?.expect("Delete keyword vanished"); // Consume the token.
                let val = self.parse(cursor, interner)?;

                if cursor.strict_mode() {
                    if let Node::Identifier(_) = val {
                        return Err(ParseError::lex(LexError::Syntax(
                            "Delete <variable> statements not allowed in strict mode".into(),
                            token_start,
                        )));
                    }
                }

                Ok(node::UnaryOp::new(UnaryOp::Delete, val).into())
            }
            TokenKind::Keyword(Keyword::Void) => {
                cursor.next(interner)?.expect("Void keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Void, self.parse(cursor, interner)?).into())
            }
            TokenKind::Keyword(Keyword::TypeOf) => {
                cursor.next(interner)?.expect("TypeOf keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::TypeOf, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Add) => {
                cursor.next(interner)?.expect("+ token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Plus, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Sub) => {
                cursor.next(interner)?.expect("- token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Minus, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Neg) => {
                cursor.next(interner)?.expect("~ token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Tilde, self.parse(cursor, interner)?).into())
            }
            TokenKind::Punctuator(Punctuator::Not) => {
                cursor.next(interner)?.expect("! token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Not, self.parse(cursor, interner)?).into())
            }
            _ => UpdateExpression::new(self.allow_yield, self.allow_await).parse(cursor, interner),
        }
    }
}
