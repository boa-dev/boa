//! Call expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions
//! [spec]: https://tc39.es/ecma262/#prod-CallExpression

use super::arguments::Arguments;
use crate::{
    syntax::{
        ast::{
            node::{
                field::{GetConstField, GetField},
                Call,
            },
            Node, Punctuator, Span,
        },
        lexer::TokenKind,
        parser::{
            expression::{left_hand_side::template::TaggedTemplateLiteral, Expression},
            AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Parses a call expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CallExpression
#[derive(Debug)]
pub(super) struct CallExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    first_member_expr: Node,
}

impl CallExpression {
    /// Creates a new `CallExpression` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, first_member_expr: Node) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            first_member_expr,
        }
    }
}

impl<R> TokenParser<R> for CallExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("CallExpression", "Parsing");

        let token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        let mut lhs = if token.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) {
            let (args, span) = Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
            Node::new(Call::new(self.first_member_expr, args), span)
        } else {
            let next_token = cursor.next()?.expect("token vanished");
            return Err(ParseError::expected(
                vec![TokenKind::Punctuator(Punctuator::OpenParen)],
                next_token,
                "call expression",
            ));
        };

        while let Some(tok) = cursor.peek(0)? {
            let token = tok.clone();
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let (args, span) =
                        Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
                    lhs = Node::new(Call::new(lhs, args), span);
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    let span_start = cursor.next()?.ok_or(ParseError::AbruptEnd)?.span().start(); // We move the parser forward.
                    let next_token = cursor.next()?.ok_or(ParseError::AbruptEnd)?;

                    let span = Span::new(span_start, next_token.span().end());

                    match next_token.kind() {
                        TokenKind::Identifier(name) => {
                            lhs = Node::new(GetConstField::new(lhs, name.clone()), span);
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = Node::new(GetConstField::new(lhs, kw.to_string()), span);
                        }
                        _ => {
                            return Err(ParseError::expected(
                                vec![TokenKind::identifier("identifier")],
                                token,
                                "call expression",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let span_start = token.span().start();

                    let _ = cursor.next()?.ok_or(ParseError::AbruptEnd)?; // We move the parser.
                    let idx =
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                    let span_end = cursor
                        .expect(Punctuator::CloseBracket, "call expression")?
                        .span()
                        .end();

                    let span = Span::new(span_start, span_end);
                    lhs = Node::new(GetField::new(lhs, idx), span);
                }
                TokenKind::TemplateNoSubstitution { .. } | TokenKind::TemplateMiddle { .. } => {
                    lhs = TaggedTemplateLiteral::new(
                        self.allow_yield,
                        self.allow_await,
                        tok.span().start(),
                        lhs,
                    )
                    .parse(cursor)?;
                }
                _ => break,
            }
        }
        Ok(lhs)
    }
}
