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
    lexer::TokenKind,
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
        let mut has_trailing_comma_spread = false;
        let mut next_comma = false;
        let mut last_spread = false;

        loop {
            let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
            match token.kind() {
                TokenKind::Punctuator(Punctuator::CloseBracket) => {
                    cursor.next(interner).expect("token disappeared");
                    break;
                }
                TokenKind::Punctuator(Punctuator::Comma) if next_comma => {
                    cursor.next(interner).expect("token disappeared");

                    if last_spread {
                        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                        if token.kind() == &TokenKind::Punctuator(Punctuator::CloseBracket) {
                            has_trailing_comma_spread = true;
                        }
                    }

                    next_comma = false;
                }
                TokenKind::Punctuator(Punctuator::Comma) => {
                    cursor.next(interner).expect("token disappeared");
                    elements.push(Node::Empty);
                }
                TokenKind::Punctuator(Punctuator::Spread) if next_comma => {
                    return Err(ParseError::unexpected(
                        token.to_string(interner),
                        token.span(),
                        "expected comma or end of array",
                    ));
                }
                TokenKind::Punctuator(Punctuator::Spread) => {
                    cursor.next(interner).expect("token disappeared");
                    let node =
                        AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                    elements.push(Spread::new(node).into());
                    next_comma = true;
                    last_spread = true;
                }
                _ if next_comma => {
                    return Err(ParseError::unexpected(
                        token.to_string(interner),
                        token.span(),
                        "expected comma or end of array",
                    ));
                }
                _ => {
                    let node =
                        AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                    elements.push(node);
                    next_comma = true;
                    last_spread = false;
                }
            }
        }

        if last_spread {
            if let Some(Node::Empty) = elements.last() {
                has_trailing_comma_spread = true;
            }
        }

        Ok(ArrayDecl::new(elements, has_trailing_comma_spread))
    }
}
