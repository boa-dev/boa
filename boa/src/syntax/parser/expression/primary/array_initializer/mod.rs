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
            node::{ArrayDecl, Node, Spread},
            Const, Punctuator,
        },
        parser::{expression::AssignmentExpression, Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};

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
pub(super) struct ArrayLiteral<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for ArrayLiteral<YIELD, AWAIT>
where
    R: Read,
{
    type Output = ArrayDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ArrayLiteral", "Parsing");
        let mut elements = Vec::new();

        loop {
            // TODO: Support all features.
            while cursor.next_if(Punctuator::Comma)?.is_some() {
                elements.push(Node::Const(Const::Undefined));
            }

            if cursor.next_if(Punctuator::CloseBracket)?.is_some() {
                break;
            }

            let _ = cursor.peek(0)?.ok_or(ParseError::AbruptEnd); // Check that there are more tokens to read.

            if cursor.next_if(Punctuator::Spread)?.is_some() {
                let node = AssignmentExpression::<true, YIELD, AWAIT>.parse(cursor)?;
                elements.push(Spread::new(node).into());
            } else {
                elements.push(AssignmentExpression::<true, YIELD, AWAIT>.parse(cursor)?);
            }
            cursor.next_if(Punctuator::Comma)?;
        }

        Ok(elements.into())
    }
}
