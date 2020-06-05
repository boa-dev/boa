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
            Keyword, Punctuator, TokenKind,
        },
        parser::{
            expression::{primary::PrimaryExpression, Expression},
            AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

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

impl TokenParser for MemberExpression {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("MemberExpression", "Parsing");
        let mut lhs = if cursor.peek(0).ok_or(ParseError::AbruptEnd)?.kind
            == TokenKind::Keyword(Keyword::New)
        {
            let _ = cursor.next().expect("keyword disappeared");
            let lhs = self.parse(cursor)?;
            let args = Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
            let call_node = Call::new(lhs, args);

            Node::from(New::from(call_node))
        } else {
            PrimaryExpression::new(self.allow_yield, self.allow_await).parse(cursor)?
        };
        while let Some(tok) = cursor.peek(0) {
            match &tok.kind {
                TokenKind::Punctuator(Punctuator::Dot) => {
                    let _ = cursor.next().ok_or(ParseError::AbruptEnd)?; // We move the cursor forward.
                    match &cursor.next().ok_or(ParseError::AbruptEnd)?.kind {
                        TokenKind::Identifier(name) => {
                            lhs = GetConstField::new(lhs, name.clone()).into()
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = GetConstField::new(lhs, kw.to_string()).into()
                        }
                        _ => {
                            return Err(ParseError::expected(
                                vec![TokenKind::identifier("identifier")],
                                tok.clone(),
                                "member expression",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let _ = cursor.next().ok_or(ParseError::AbruptEnd)?; // We move the cursor forward.
                    let idx =
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                    cursor.expect(Punctuator::CloseBracket, "member expression")?;
                    lhs = GetField::new(lhs, idx).into();
                }
                _ => break,
            }
        }

        Ok(lhs)
    }
}
