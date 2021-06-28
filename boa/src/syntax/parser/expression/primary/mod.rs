//! Primary expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Primary_expressions
//! [spec]: https://tc39.es/ecma262/#prod-PrimaryExpression

mod array_initializer;
mod async_function_expression;
mod function_expression;
mod object_initializer;
mod template;
#[cfg(test)]
mod tests;

use self::{
    array_initializer::ArrayLiteral, async_function_expression::AsyncFunctionExpression,
    function_expression::FunctionExpression, object_initializer::ObjectLiteral,
};
use super::Expression;
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{
            node::{Call, Identifier, New},
            Const, Keyword, Node, NodeKind, Punctuator, Span,
        },
        lexer::{token::Numeric, InputElement, TokenKind},
        parser::{
            expression::primary::template::TemplateLiteral, AllowAwait, AllowYield, Cursor,
            ParseError, ParseResult, TokenParser,
        },
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

        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::This) => Ok(Node::new(
                NodeKind::This,
                cursor.next()?.expect("token disappeared").span(),
            )),
            TokenKind::Keyword(Keyword::Function) => {
                let (expr, span) = FunctionExpression.parse(cursor)?;
                Ok(Node::new(NodeKind::from(expr), span))
            }
            TokenKind::Keyword(Keyword::Async) => {
                let (expr, span) = AsyncFunctionExpression::new(self.allow_yield).parse(cursor)?;
                Ok(Node::new(NodeKind::from(expr), span))
            }
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                let span_start = cursor.next()?.expect("token disappeared").span().start();
                cursor.set_goal(InputElement::RegExp);
                let expr =
                    Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                let span_end = cursor
                    .expect(Punctuator::CloseParen, "primary expression")?
                    .span()
                    .end();

                Ok(Node::new(expr.into_kind(), Span::new(span_start, span_end)))
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                cursor.set_goal(InputElement::RegExp);
                let (expr, span) =
                    ArrayLiteral::new(self.allow_yield, self.allow_await).parse(cursor)?;

                Ok(Node::new(expr, span))
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                cursor.set_goal(InputElement::RegExp);
                let (expr, span) =
                    ObjectLiteral::new(self.allow_yield, self.allow_await).parse(cursor)?;

                Ok(Node::new(expr, span))
            }
            TokenKind::BooleanLiteral(boolean) => Ok(Node::new(Const::from(*boolean), tok.span())),
            TokenKind::NullLiteral => Ok(Node::new(Const::Null, tok.span())),
            TokenKind::Identifier(ident) => {
                Ok(Node::new(Identifier::from(ident.as_ref()), tok.span()))
            } // TODO: IdentifierReference
            TokenKind::StringLiteral(s) => Ok(Node::new(Const::from(s.as_ref()), tok.span())),
            TokenKind::TemplateNoSubstitution(template_string) => Ok(Node::new(
                Const::from(template_string.to_owned_cooked().map_err(ParseError::lex)?),
                tok.span(),
            )),
            TokenKind::NumericLiteral(Numeric::Integer(num)) => {
                Ok(Node::new(Const::from(*num), tok.span()))
            }
            TokenKind::NumericLiteral(Numeric::Rational(num)) => {
                Ok(Node::new(Const::from(*num), tok.span()))
            }
            TokenKind::NumericLiteral(Numeric::BigInt(num)) => {
                Ok(Node::new(Const::from(num.clone()), tok.span()))
            }
            TokenKind::RegularExpressionLiteral(body, flags) => Ok(Node::new(
                // FIXME: properly use flags and body spans, maybe a new AST node.
                New::from(Call::new(
                    Node::new(Identifier::from("RegExp"), tok.span()),
                    vec![
                        Node::new(Const::from(body.as_ref()), tok.span()),
                        Node::new(Const::from(flags.to_string()), tok.span()),
                    ],
                )),
                tok.span(),
            )),
            TokenKind::Punctuator(Punctuator::Div) => {
                let tok = cursor.lex_regex(tok.span().start())?;

                if let TokenKind::RegularExpressionLiteral(body, flags) = tok.kind() {
                    // FIXME: properly use flags and body spans, maybe a new AST node.
                    Ok(Node::new(
                        New::from(Call::new(
                            Node::new(Identifier::from("RegExp"), tok.span()),
                            vec![
                                Node::new(Const::from(body.as_ref()), tok.span()),
                                Node::new(Const::from(flags.to_string()), tok.span()),
                            ],
                        )),
                        tok.span(),
                    ))
                } else {
                    // A regex was expected and nothing else.
                    Err(ParseError::unexpected(tok, "regular expression literal"))
                }
            }
            TokenKind::TemplateMiddle(template_string) => {
                let (expr, span) = TemplateLiteral::new(
                    self.allow_yield,
                    self.allow_await,
                    tok.span().start(),
                    template_string
                        .to_owned_cooked()
                        .map_err(ParseError::lex)?
                        .as_ref(),
                )
                .parse(cursor)?;

                Ok(Node::new(expr, span))
            }
            _ => Err(ParseError::unexpected(tok.clone(), "primary expression")),
        }
    }
}
