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
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{Node, Punctuator},
        lexer::{InputElement, TokenKind},
        parser::{Cursor, ParseError, TokenParser},
    },
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
pub(super) struct LeftHandSideExpression<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for LeftHandSideExpression<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("LeftHandSIdeExpression", "Parsing");

        cursor.set_goal(InputElement::TemplateTail);

        // TODO: Implement NewExpression: new MemberExpression
        let lhs = MemberExpression::<YIELD, AWAIT>.parse(cursor)?;
        if let Some(tok) = cursor.peek(0)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::OpenParen) {
                return CallExpression::<YIELD, AWAIT>::new(lhs).parse(cursor);
            }
        }
        Ok(lhs)
    }
}
