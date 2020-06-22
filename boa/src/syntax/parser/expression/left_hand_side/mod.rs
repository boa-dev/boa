//! Left hand side expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Left-hand-side_expressions
//! [spec]: https://tc39.es/ecma262/#sec-left-hand-side-expressions

mod arguments;
mod call;
mod member;

use self::{call::CallExpression, member::MemberExpression};
use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{Node, Punctuator},
        parser::{AllowAwait, AllowYield, ParseResult, Cursor, TokenParser},
    },
    BoaProfiler,
};

use std::io::Read;

/// Parses a left hand side expression.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Left-hand-side_expressions
/// [spec]: https://tc39.es/ecma262/#prod-LeftHandSideExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct LeftHandSideExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LeftHandSideExpression {
    /// Creates a new `LeftHandSideExpression` parser.
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

impl<R> TokenParser<R> for LeftHandSideExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("LeftHandSIdeExpression", "Parsing");
        // TODO: Implement NewExpression: new MemberExpression
        let lhs = MemberExpression::new(self.allow_yield, self.allow_await).parse(cursor)?;
        match cursor.peek(0) {
            Some(ref tok) if tok.kind == TokenKind::Punctuator(Punctuator::OpenParen) => {
                CallExpression::new(self.allow_yield, self.allow_await, lhs).parse(cursor)
            }
            _ => Ok(lhs), // TODO: is this correct?
        }
    }
}
