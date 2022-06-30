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

use crate::syntax::{
    ast::{node::SuperCall, Keyword, Node, Punctuator},
    lexer::{InputElement, TokenKind},
    parser::{
        expression::left_hand_side::{
            arguments::Arguments, call::CallExpression, member::MemberExpression,
        },
        AllowAwait, AllowYield, Cursor, ParseResult, TokenParser,
    },
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
pub(in crate::syntax::parser) struct LeftHandSideExpression {
    name: Option<Sym>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl LeftHandSideExpression {
    /// Creates a new `LeftHandSideExpression` parser.
    pub(in crate::syntax::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
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

        if let Some(next) = cursor.peek(0, interner)? {
            if let TokenKind::Keyword((Keyword::Super, _)) = next.kind() {
                if let Some(next) = cursor.peek(1, interner)? {
                    if next.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) {
                        cursor.next(interner).expect("token disappeared");
                        let args = Arguments::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                        return Ok(SuperCall::new(args).into());
                    }
                }
            }
        }

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
