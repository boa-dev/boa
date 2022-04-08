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
use boa_interner::{Interner, Sym};
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
    name: Option<Sym>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl UnaryExpression {
    /// Creates a new `UnaryExpression` parser.
    pub(in crate::syntax::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
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
            TokenKind::Keyword((Keyword::Delete | Keyword::Void | Keyword::TypeOf, true)) => Err(
                ParseError::general("Keyword must not contain escaped characters", token_start),
            ),
            TokenKind::Keyword((Keyword::Delete, false)) => {
                cursor.next(interner)?.expect("Delete keyword vanished");
                let position = cursor
                    .peek(0, interner)?
                    .ok_or(ParseError::AbruptEnd)?
                    .span()
                    .start();
                let val = self.parse(cursor, interner)?;

                match val {
                    Node::Identifier(_) if cursor.strict_mode() => {
                        return Err(ParseError::lex(LexError::Syntax(
                            "Delete <variable> statements not allowed in strict mode".into(),
                            token_start,
                        )));
                    }
                    Node::GetPrivateField(_) => {
                        return Err(ParseError::general(
                            "private fields can not be deleted",
                            position,
                        ));
                    }
                    _ => {}
                }

                Ok(node::UnaryOp::new(UnaryOp::Delete, val).into())
            }
            TokenKind::Keyword((Keyword::Void, false)) => {
                cursor.next(interner)?.expect("Void keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Void, self.parse(cursor, interner)?).into())
            }
            TokenKind::Keyword((Keyword::TypeOf, false)) => {
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
            _ => UpdateExpression::new(self.name, self.allow_yield, self.allow_await)
                .parse(cursor, interner),
        }
    }
}
