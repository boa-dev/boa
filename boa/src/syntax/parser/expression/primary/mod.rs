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
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{
            node::{Call, Identifier, New, Node},
            Const, Keyword, Punctuator,
        },
        lexer::{token::Numeric, InputElement, TokenKind},
        parser::{AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
    },
};
pub(in crate::syntax::parser) use object_initializer::Initializer;

use std::io::Read;

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

impl<R> TokenParser<R> for PrimaryExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("PrimaryExpression", "Parsing");

        let tok = cursor.next()?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::This) => Ok(Node::This),
            TokenKind::Keyword(Keyword::Function) => {
                FunctionExpression.parse(cursor).map(Node::from)
            }
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                cursor.set_goal(InputElement::RegExp);
                let expr =
                    Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "primary expression")?;
                Ok(expr)
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                cursor.set_goal(InputElement::RegExp);
                ArrayLiteral::new(self.allow_yield, self.allow_await)
                    .parse(cursor)
                    .map(Node::ArrayDecl)
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                cursor.set_goal(InputElement::RegExp);
                Ok(ObjectLiteral::new(self.allow_yield, self.allow_await)
                    .parse(cursor)?
                    .into())
            }
            TokenKind::BooleanLiteral(boolean) => Ok(Const::from(*boolean).into()),
            TokenKind::NullLiteral => Ok(Const::Null.into()),
            TokenKind::Identifier(ident) => Ok(Identifier::from(ident.as_ref()).into()), // TODO: IdentifierReference
            TokenKind::StringLiteral(s) => Ok(Const::from(s.as_ref()).into()),
            TokenKind::NumericLiteral(Numeric::Integer(num)) => Ok(Const::from(*num).into()),
            TokenKind::NumericLiteral(Numeric::Rational(num)) => Ok(Const::from(*num).into()),
            TokenKind::NumericLiteral(Numeric::BigInt(num)) => Ok(Const::from(num.clone()).into()),
            TokenKind::RegularExpressionLiteral(body, flags) => {
                Ok(Node::from(New::from(Call::new(
                    Identifier::from("RegExp"),
                    vec![
                        Const::from(body.as_ref()).into(),
                        Const::from(flags.to_string()).into(),
                    ],
                ))))
            }
            TokenKind::Punctuator(Punctuator::Div) => {
                let tok = cursor.lex_regex(tok.span().start())?;

                if let TokenKind::RegularExpressionLiteral(body, flags) = tok.kind() {
                    Ok(Node::from(New::from(Call::new(
                        Identifier::from("RegExp"),
                        vec![
                            Const::from(body.as_ref()).into(),
                            Const::from(flags.to_string()).into(),
                        ],
                    ))))
                } else {
                    // A regex was expected and nothing else.
                    Err(ParseError::unexpected(tok, "regular expression literal"))
                }
            }
            _ => Err(ParseError::unexpected(tok.clone(), "primary expression")),
        }
    }
}
