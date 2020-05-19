//! Primary expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Primary_expressions
//! [spec]: https://tc39.es/ecma262/#prod-PrimaryExpression

mod array_initializer;
mod function_expression;
mod object_initializer;
#[cfg(test)]
mod tests;

use self::{
    array_initializer::ArrayLiteral, function_expression::FunctionExpression,
    object_initializer::ObjectLiteral,
};
use super::Expression;
use crate::syntax::{
    ast::{
        constant::Const, keyword::Keyword, node::Node, punc::Punctuator, token::NumericLiteral,
        token::TokenKind,
    },
    parser::{AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
};
pub(in crate::syntax::parser) use object_initializer::Initializer;

/// Parses a primary expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Primary_expressions
/// [spec]: https://tc39.es/ecma262/#prod-PrimaryExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct PrimaryExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PrimaryExpression {
    /// Creates a new `PrimaryExpression` parser.
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

impl TokenParser for PrimaryExpression {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let tok = cursor.next().ok_or(ParseError::AbruptEnd)?;

        match &tok.kind {
            TokenKind::Keyword(Keyword::This) => Ok(Node::This),
            // TokenKind::Keyword(Keyword::Arguments) => Ok(Node::new(NodeBase::Arguments, tok.pos)),
            TokenKind::Keyword(Keyword::Function) => FunctionExpression.parse(cursor),
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                let expr =
                    Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "primary expression")?;
                Ok(expr)
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                ArrayLiteral::new(self.allow_yield, self.allow_await).parse(cursor)
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                ObjectLiteral::new(self.allow_yield, self.allow_await).parse(cursor)
            }
            TokenKind::BooleanLiteral(boolean) => Ok(Node::const_node(*boolean)),
            // TODO: ADD TokenKind::UndefinedLiteral
            TokenKind::Identifier(ref i) if i == "undefined" => Ok(Node::Const(Const::Undefined)),
            TokenKind::NullLiteral => Ok(Node::Const(Const::Null)),
            TokenKind::Identifier(ident) => Ok(Node::local(ident)), // TODO: IdentifierReference
            TokenKind::StringLiteral(s) => Ok(Node::const_node(s)),
            TokenKind::NumericLiteral(NumericLiteral::Integer(num)) => Ok(Node::const_node(*num)),
            TokenKind::NumericLiteral(NumericLiteral::Rational(num)) => Ok(Node::const_node(*num)),
            TokenKind::NumericLiteral(NumericLiteral::BigInt(num)) => {
                Ok(Node::const_node(num.clone()))
            }
            TokenKind::RegularExpressionLiteral(body, flags) => Ok(Node::new(Node::call(
                Node::local("RegExp"),
                vec![Node::const_node(body), Node::const_node(flags)],
            ))),
            _ => Err(ParseError::Unexpected(
                tok.clone(),
                Some("primary expression"),
            )),
        }
    }
}
