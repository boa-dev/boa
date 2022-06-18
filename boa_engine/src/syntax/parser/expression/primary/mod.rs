//! Primary expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Primary_expressions
//! [spec]: https://tc39.es/ecma262/#prod-PrimaryExpression

#[cfg(test)]
mod tests;

mod array_initializer;
mod async_function_expression;
mod async_generator_expression;
mod class_expression;
mod function_expression;
mod generator_expression;
mod template;

pub(in crate::syntax::parser) mod object_initializer;

use self::{
    array_initializer::ArrayLiteral, async_function_expression::AsyncFunctionExpression,
    async_generator_expression::AsyncGeneratorExpression, class_expression::ClassExpression,
    function_expression::FunctionExpression, generator_expression::GeneratorExpression,
    object_initializer::ObjectLiteral,
};
use crate::syntax::{
    ast::{
        node::{Call, Identifier, New, Node},
        Const, Keyword, Punctuator,
    },
    lexer::{token::Numeric, InputElement, TokenKind},
    parser::{
        expression::{
            identifiers::IdentifierReference, primary::template::TemplateLiteral, Expression,
        },
        AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

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
    name: Option<Sym>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PrimaryExpression {
    /// Creates a new `PrimaryExpression` parser.
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

impl<R> TokenParser<R> for PrimaryExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("PrimaryExpression", "Parsing");

        // TODO: tok currently consumes the token instead of peeking, so the token
        // isn't passed and consumed by parsers according to spec (EX: GeneratorExpression)
        let tok = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword((Keyword::This | Keyword::Async, true)) => Err(ParseError::general(
                "Keyword must not contain escaped characters",
                tok.span().start(),
            )),
            TokenKind::Keyword((Keyword::This, false)) => {
                cursor.next(interner).expect("token disappeared");
                Ok(Node::This)
            }
            TokenKind::Keyword((Keyword::Function, _)) => {
                cursor.next(interner).expect("token disappeared");
                let next_token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                if next_token.kind() == &TokenKind::Punctuator(Punctuator::Mul) {
                    GeneratorExpression::new(self.name)
                        .parse(cursor, interner)
                        .map(Node::from)
                } else {
                    FunctionExpression::new(self.name)
                        .parse(cursor, interner)
                        .map(Node::from)
                }
            }
            TokenKind::Keyword((Keyword::Class, _)) => {
                cursor.next(interner).expect("token disappeared");
                ClassExpression::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
            }
            TokenKind::Keyword((Keyword::Async, false)) => {
                cursor.next(interner).expect("token disappeared");
                let mul_peek = cursor.peek(1, interner)?.ok_or(ParseError::AbruptEnd)?;
                if mul_peek.kind() == &TokenKind::Punctuator(Punctuator::Mul) {
                    AsyncGeneratorExpression::new(self.name)
                        .parse(cursor, interner)
                        .map(Node::from)
                } else {
                    AsyncFunctionExpression::new(self.name, self.allow_yield)
                        .parse(cursor, interner)
                        .map(Node::from)
                }
            }
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                cursor.next(interner).expect("token disappeared");
                cursor.set_goal(InputElement::RegExp);
                let expr = Expression::new(self.name, true, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                cursor.expect(Punctuator::CloseParen, "primary expression", interner)?;
                Ok(expr)
            }
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                cursor.next(interner).expect("token disappeared");
                cursor.set_goal(InputElement::RegExp);
                ArrayLiteral::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
                    .map(Node::ArrayDecl)
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                cursor.next(interner).expect("token disappeared");
                cursor.set_goal(InputElement::RegExp);
                Ok(ObjectLiteral::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
                    .into())
            }
            TokenKind::BooleanLiteral(boolean) => {
                let node = Const::from(*boolean).into();
                cursor.next(interner).expect("token disappeared");
                Ok(node)
            }
            TokenKind::NullLiteral => {
                cursor.next(interner).expect("token disappeared");
                Ok(Const::Null.into())
            }
            TokenKind::Identifier(_)
            | TokenKind::Keyword((Keyword::Let | Keyword::Yield | Keyword::Await, _)) => {
                IdentifierReference::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)
                    .map(Node::from)
            }
            TokenKind::StringLiteral(lit) => {
                let node = Const::from(*lit).into();
                cursor.next(interner).expect("token disappeared");
                Ok(node)
            }
            TokenKind::TemplateNoSubstitution(template_string) => {
                let node = Const::from(
                    template_string
                        .to_owned_cooked(interner)
                        .map_err(ParseError::lex)?,
                )
                .into();
                cursor.next(interner).expect("token disappeared");
                Ok(node)
            }
            TokenKind::NumericLiteral(Numeric::Integer(num)) => {
                let node = Const::from(*num).into();
                cursor.next(interner).expect("token disappeared");
                Ok(node)
            }
            TokenKind::NumericLiteral(Numeric::Rational(num)) => {
                let node = Const::from(*num).into();
                cursor.next(interner).expect("token disappeared");
                Ok(node)
            }
            TokenKind::NumericLiteral(Numeric::BigInt(num)) => {
                let node = Const::from(num.clone()).into();
                cursor.next(interner).expect("token disappeared");
                Ok(node)
            }
            TokenKind::RegularExpressionLiteral(body, flags) => {
                let node = Node::from(New::from(Call::new(
                    Identifier::new(Sym::REGEXP),
                    vec![Const::from(*body).into(), Const::from(*flags).into()],
                )));
                cursor.next(interner).expect("token disappeared");
                Ok(node)
            }
            TokenKind::Punctuator(Punctuator::Div) => {
                let position = tok.span().start();
                cursor.next(interner).expect("token disappeared");
                let tok = cursor.lex_regex(position, interner)?;

                if let TokenKind::RegularExpressionLiteral(body, flags) = *tok.kind() {
                    Ok(Node::from(New::from(Call::new(
                        Identifier::new(Sym::REGEXP),
                        vec![Const::from(body).into(), Const::from(flags).into()],
                    ))))
                } else {
                    // A regex was expected and nothing else.
                    Err(ParseError::unexpected(
                        tok.to_string(interner),
                        tok.span(),
                        "regular expression literal",
                    ))
                }
            }
            TokenKind::TemplateMiddle(template_string) => {
                let parser = TemplateLiteral::new(
                    self.allow_yield,
                    self.allow_await,
                    tok.span().start(),
                    template_string
                        .to_owned_cooked(interner)
                        .map_err(ParseError::lex)?,
                );
                cursor.next(interner).expect("token disappeared");
                parser.parse(cursor, interner).map(Node::TemplateLit)
            }
            _ => Err(ParseError::unexpected(
                tok.to_string(interner),
                tok.span(),
                "primary expression",
            )),
        }
    }
}
