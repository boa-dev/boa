//! Member expression parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-MemberExpression

use super::arguments::Arguments;
use crate::{
    syntax::{
        ast::{
            node::{
                field::{GetConstField, GetField},
                Call, New,
            },
            Keyword, Node, Punctuator, Span,
        },
        lexer::TokenKind,
        parser::{
            expression::{
                left_hand_side::template::TaggedTemplateLiteral, primary::PrimaryExpression,
                Expression,
            },
            AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Parses a member expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-MemberExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct MemberExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl MemberExpression {
    /// Creates a new `MemberExpression` parser.
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

impl<R> TokenParser<R> for MemberExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("MemberExpression", "Parsing");

        let mut lhs = if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Keyword(Keyword::New)
        {
            let start_span = cursor
                .next()?
                .expect("new keyword disappeared")
                .span()
                .start();
            let lhs = self.parse(cursor)?;
            let (args, args_span) =
                Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
            let call_node = Call::new(lhs, args);

            let span = Span::new(start_span, args_span.end());
            Node::new(New::from(call_node), span)
        } else {
            PrimaryExpression::new(self.allow_yield, self.allow_await).parse(cursor)?
        };
        while let Some(tok) = cursor.peek(0)? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Dot) => {
                    let span_start = cursor
                        .next()?
                        .expect("dot punctuator token disappeared")
                        .span()
                        .start(); // We move the parser forward.

                    let token = cursor.next()?.ok_or(ParseError::AbruptEnd)?;

                    let span = Span::new(span_start, token.span().end());

                    match token.kind() {
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
                                "member expression",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let span_start = cursor
                        .next()?
                        .expect("open bracket punctuator token disappeared")
                        .span()
                        .start(); // We move the parser forward.

                    let idx =
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                    let span_end = cursor
                        .expect(Punctuator::CloseBracket, "member expression")?
                        .span()
                        .end();

                    lhs = Node::new(GetField::new(lhs, idx), Span::new(span_start, span_end));
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
