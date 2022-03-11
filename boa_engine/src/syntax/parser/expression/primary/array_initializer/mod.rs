//! Array initializer parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
//! [spec]: https://tc39.es/ecma262/#sec-array-initializer

#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{
        node::{ArrayDecl, Node, Spread},
        Punctuator,
    },
    parser::{
        expression::AssignmentExpression, AllowAwait, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Parses an array literal.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
/// [spec]: https://tc39.es/ecma262/#prod-ArrayLiteral
#[derive(Debug, Clone, Copy)]
pub(super) struct ArrayLiteral {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ArrayLiteral {
    /// Creates a new `ArrayLiteral` parser.
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

impl<R> TokenParser<R> for ArrayLiteral
where
    R: Read,
{
    type Output = ArrayDecl;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("ArrayLiteral", "Parsing");
        let mut elements = Vec::new();

        loop {
            // TODO: Support all features.
            while cursor.next_if(Punctuator::Comma, interner)?.is_some() {
                elements.push(Node::Empty);
            }

            if cursor
                .next_if(Punctuator::CloseBracket, interner)?
                .is_some()
            {
                break;
            }

            let _next = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd); // Check that there are more tokens to read.

            if cursor.next_if(Punctuator::Spread, interner)?.is_some() {
                let node =
                    AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                elements.push(Spread::new(node).into());
            } else {
                elements.push(
                    AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?,
                );
            }
            cursor.next_if(Punctuator::Comma, interner)?;
        }

        Ok(elements.into())
    }
}
