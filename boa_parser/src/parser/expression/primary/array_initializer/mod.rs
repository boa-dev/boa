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
    lexer::TokenKind,
    parser::{
        expression::AssignmentExpression, AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult,
        TokenParser,
    },
    Error,
};
use boa_ast::{
    expression::{literal, Spread},
    Punctuator,
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
    type Output = literal::ArrayLiteral;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ArrayLiteral", "Parsing");
        let mut elements = Vec::new();
        let mut has_trailing_comma_spread = false;
        let mut next_comma = false;
        let mut last_spread = false;

        loop {
            let token = cursor.peek(0, interner).or_abrupt()?;
            match token.kind() {
                TokenKind::Punctuator(Punctuator::CloseBracket) => {
                    cursor.advance(interner);
                    break;
                }
                TokenKind::Punctuator(Punctuator::Comma) if next_comma => {
                    cursor.advance(interner);

                    if last_spread {
                        let token = cursor.peek(0, interner).or_abrupt()?;
                        if token.kind() == &TokenKind::Punctuator(Punctuator::CloseBracket) {
                            has_trailing_comma_spread = true;
                        }
                    }

                    next_comma = false;
                }
                TokenKind::Punctuator(Punctuator::Comma) => {
                    cursor.advance(interner);
                    elements.push(None);
                }
                TokenKind::Punctuator(Punctuator::Spread) if next_comma => {
                    return Err(Error::unexpected(
                        token.to_string(interner),
                        token.span(),
                        "expected comma or end of array",
                    ));
                }
                TokenKind::Punctuator(Punctuator::Spread) => {
                    cursor.advance(interner);
                    let node =
                        AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                    elements.push(Some(Spread::new(node).into()));
                    next_comma = true;
                    last_spread = true;
                }
                _ if next_comma => {
                    return Err(Error::unexpected(
                        token.to_string(interner),
                        token.span(),
                        "expected comma or end of array",
                    ));
                }
                _ => {
                    let expr =
                        AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                    elements.push(Some(expr));
                    next_comma = true;
                    last_spread = false;
                }
            }
        }

        if last_spread {
            if let Some(None) = elements.last() {
                has_trailing_comma_spread = true;
            }
        }

        Ok(literal::ArrayLiteral::new(
            elements,
            has_trailing_comma_spread,
        ))
    }
}
