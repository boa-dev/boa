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
mod template;

use self::{call::CallExpression, member::MemberExpression};
use crate::syntax::{
    ast::{Node, Punctuator},
    lexer::{InputElement, TokenKind},
    parser::{AllowAwait, AllowYield, Cursor, ParseResult, TokenParser},
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
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
    name: Option<Sym>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LeftHandSideExpression {
    /// Creates a new `LeftHandSideExpression` parser.
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

impl<R> TokenParser<R> for LeftHandSideExpression
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("LeftHandSIdeExpression", "Parsing");

        cursor.set_goal(InputElement::TemplateTail);

        // TODO: Implement NewExpression: new MemberExpression
        let lhs = MemberExpression::new(self.name, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;
        if let Some(tok) = cursor.peek(0, interner)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) {
                return CallExpression::new(self.allow_yield, self.allow_await, lhs)
                    .parse(cursor, interner);
            }
        }
        Ok(lhs)
    }
}
