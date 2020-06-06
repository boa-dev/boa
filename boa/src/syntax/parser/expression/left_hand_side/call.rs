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
            node::{Call, Node},
            Punctuator, TokenKind,
        },
        parser::{
            expression::Expression, AllowAwait, AllowYield, Cursor, ParseError, ParseResult,
            TokenParser,
        },
    },
    BoaProfiler,
};

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

impl TokenParser for CallExpression {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("CallExpression", "Parsing");
        let mut lhs = match cursor.peek(0) {
            Some(tk) if tk.kind == TokenKind::Punctuator(Punctuator::OpenParen) => {
                let args = Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
                Node::from(Call::new(self.first_member_expr, args))
            }
            _ => {
                let next_token = cursor.next().ok_or(ParseError::AbruptEnd)?;
                return Err(ParseError::expected(
                    vec![TokenKind::Punctuator(Punctuator::OpenParen)],
                    next_token.clone(),
                    "call expression",
                ));
            }
        };

        while let Some(tok) = cursor.peek(0) {
            match tok.kind {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let args = Arguments::new(self.allow_yield, self.allow_await).parse(cursor)?;
                    lhs = Node::from(Call::new(lhs, args));
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    let _ = cursor.next().ok_or(ParseError::AbruptEnd)?; // We move the cursor.
                    match &cursor.next().ok_or(ParseError::AbruptEnd)?.kind {
                        TokenKind::Identifier(name) => {
                            lhs = Node::get_const_field(lhs, name.clone());
                        }
                        TokenKind::Keyword(kw) => {
                            lhs = Node::get_const_field(lhs, kw.to_string());
                        }
                        _ => {
                            return Err(ParseError::expected(
                                vec![TokenKind::identifier("identifier")],
                                tok.clone(),
                                "call expression",
                            ));
                        }
                    }
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    let _ = cursor.next().ok_or(ParseError::AbruptEnd)?; // We move the cursor.
                    let idx =
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                    cursor.expect(Punctuator::CloseBracket, "call expression")?;
                    lhs = Node::get_field(lhs, idx);
                }
                _ => break,
            }
        }
        Ok(lhs)
    }
}
