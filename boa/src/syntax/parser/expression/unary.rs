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
        lexer::{Error as LexError, TokenKind},
        parser::{
            expression::update::UpdateExpression, Cursor, ParseError, ParseResult, TokenParser,
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
pub(in crate::syntax::parser) struct UnaryExpression<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for UnaryExpression<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("UnaryExpression", "Parsing");

        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
        let token_start = tok.span().start();
        match tok.kind() {
            TokenKind::Keyword(Keyword::Delete) => {
                cursor.next()?.expect("Delete keyword vanished"); // Consume the token.
                let val = self.parse(cursor)?;

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
                cursor.next()?.expect("Void keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Void, self.parse(cursor)?).into())
            }
            TokenKind::Keyword(Keyword::TypeOf) => {
                cursor.next()?.expect("TypeOf keyword vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::TypeOf, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Add) => {
                cursor.next()?.expect("+ token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Plus, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Sub) => {
                cursor.next()?.expect("- token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Minus, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Neg) => {
                cursor.next()?.expect("~ token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Tilde, self.parse(cursor)?).into())
            }
            TokenKind::Punctuator(Punctuator::Not) => {
                cursor.next()?.expect("! token vanished"); // Consume the token.
                Ok(node::UnaryOp::new(UnaryOp::Not, self.parse(cursor)?).into())
            }
            _ => UpdateExpression::<YIELD, AWAIT>.parse(cursor),
        }
    }
}
