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
            node::{ArrayDecl, Spread},
            Const, Node, Punctuator, Span,
        },
        parser::{
            expression::AssignmentExpression, AllowAwait, AllowYield, Cursor, ParseError,
            TokenParser,
        },
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
    type Output = (ArrayDecl, Span);

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ArrayLiteral", "Parsing");

        let span_start = cursor
            .expect(Punctuator::OpenBracket, "array literal")?
            .span()
            .start();

        let mut elements = Vec::new();

        let span_end = loop {
            // TODO: Support all features.
            while let Some(tok) = cursor.next_if(Punctuator::Comma)? {
                elements.push(Node::new(Const::Undefined, tok.span()));
            }

            if let Some(tok) = cursor.next_if(Punctuator::CloseBracket)? {
                break tok.span().end();
            }

            let _ = cursor.peek(0)?.ok_or(ParseError::AbruptEnd); // Check that there are more tokens to read.

            if let Some(tok) = cursor.next_if(Punctuator::Spread)? {
                let span_start = tok.span().start();
                let node = AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                    .parse(cursor)?;
                elements.push(Node::new(
                    Spread::new(node),
                    Span::new(span_start, node.span().end()),
                ));
            } else {
                elements.push(
                    AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor)?,
                );
            }
            cursor.next_if(Punctuator::Comma)?;
        };

        Ok((elements.into(), Span::new(span_start, span_end)))
    }
}
