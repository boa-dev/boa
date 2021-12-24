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
                Call, New, Node,
            },
            Keyword, Punctuator,
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
    BoaProfiler, Interner,
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

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("MemberExpression", "Parsing");

        let mut lhs = if cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
            == &TokenKind::Keyword(Keyword::New)
        {
            let _ = cursor.next(interner).expect("new keyword disappeared");
            let lhs = self.parse(cursor, interner)?;
            let args = match cursor.peek(0, interner)? {
                Some(next) if next.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) => {
                    Arguments::new(self.allow_yield, self.allow_await).parse(cursor, interner)?
                }
                _ => Box::new([]),
            };
            let call_node = Call::new(lhs, args);

            Node::from(New::from(call_node))
        } else {
            PrimaryExpression::new(self.allow_yield, self.allow_await).parse(cursor, interner)?
        };
        while let Some(tok) = cursor.peek(0, interner)? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor
                        .next(interner)?
                        .expect("dot punctuator token disappeared"); // We move the parser forward.

                    let token = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;

                    match token.kind() {
                        TokenKind::Identifier(name) => {
                            lhs = GetConstField::new(
                                lhs,
                                interner.resolve(*name).expect("string disappeared"),
                            )
                            .into()
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = GetConstField::new(lhs, kw.to_string()).into()
                        }
                        _ => {
                            return Err(ParseError::expected(
                                ["identifier".to_owned()],
                                token.to_string(interner),
                                token.span(),
                                "member expression",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    cursor
                        .next(interner)?
                        .expect("open bracket punctuator token disappeared"); // We move the parser forward.
                    let idx = Expression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(Punctuator::CloseBracket, "member expression", interner)?;
                    lhs = GetField::new(lhs, idx).into();
                }
                TokenKind::TemplateNoSubstitution { .. } | TokenKind::TemplateMiddle { .. } => {
                    lhs = TaggedTemplateLiteral::new(
                        self.allow_yield,
                        self.allow_await,
                        tok.span().start(),
                        lhs,
                    )
                    .parse(cursor, interner)?;
                }
                _ => break,
            }
        }

        Ok(lhs)
    }
}
