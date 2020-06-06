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

use crate::{
    syntax::{
        ast::{
            node::{ArrayDecl, Node},
            Const, Punctuator,
        },
        parser::{
            expression::AssignmentExpression, AllowAwait, AllowYield, Cursor, ParseError,
            TokenParser,
        },
    },
    BoaProfiler,
};

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

impl TokenParser for ArrayLiteral {
    type Output = ArrayDecl;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ArrayLiteral", "Parsing");
        let mut elements = Vec::new();

        loop {
            // TODO: Support all features.
            while cursor.next_if(Punctuator::Comma).is_some() {
                elements.push(Node::Const(Const::Undefined));
            }

            if cursor.next_if(Punctuator::CloseBracket).is_some() {
                break;
            }

            let _ = cursor.peek(0).ok_or(ParseError::AbruptEnd)?; // Check that there are more tokens to read.

            if cursor.next_if(Punctuator::Spread).is_some() {
                let node = AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                    .parse(cursor)?;
                elements.push(Node::spread(node));
            } else {
                elements.push(
                    AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor)?,
                );
            }
            cursor.next_if(Punctuator::Comma);
        }

        Ok(elements.into())
    }
}
