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
            Cursor, ParseError, ParseResult, TokenParser,
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
pub(super) struct MemberExpression<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for MemberExpression<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("MemberExpression", "Parsing");

        let mut lhs = if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Keyword(Keyword::New)
        {
            let _ = cursor.next().expect("new keyword disappeared");
            let lhs = self.parse(cursor)?;
            let args = match cursor.peek(0)? {
                Some(next) if next.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) => {
                    Arguments::<YIELD, AWAIT>.parse(cursor)?
                }
                _ => Box::new([]),
            };
            let call_node = Call::new(lhs, args);

            Node::from(New::from(call_node))
        } else {
            PrimaryExpression::<YIELD, AWAIT>.parse(cursor)?
        };
        while let Some(tok) = cursor.peek(0)? {
            match tok.kind() {
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor.next()?.expect("dot punctuator token disappeared"); // We move the parser forward.

                    let token = cursor.next()?.ok_or(ParseError::AbruptEnd)?;

                    match token.kind() {
                        TokenKind::Identifier(name) => {
                            lhs = GetConstField::new(lhs, name.clone()).into()
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = GetConstField::new(lhs, kw.to_string()).into()
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
                    cursor
                        .next()?
                        .expect("open bracket punctuator token disappeared"); // We move the parser forward.
                    let idx = Expression::<true, YIELD, AWAIT>.parse(cursor)?;
                    cursor.expect(Punctuator::CloseBracket, "member expression")?;
                    lhs = GetField::new(lhs, idx).into();
                }
                TokenKind::TemplateNoSubstitution { .. } | TokenKind::TemplateMiddle { .. } => {
                    lhs = TaggedTemplateLiteral::<YIELD, AWAIT>::new(tok.span().start(), lhs)
                        .parse(cursor)?;
                }
                _ => break,
            }
        }

        Ok(lhs)
    }
}
