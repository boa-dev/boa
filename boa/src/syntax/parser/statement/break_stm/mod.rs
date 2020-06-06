//! Break expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
//! [spec]: https://tc39.es/ecma262/#sec-break-statement

#[cfg(test)]
mod tests;

use super::LabelIdentifier;
use crate::{
    syntax::{
        ast::{Keyword, Node, Punctuator, TokenKind},
        parser::{AllowAwait, AllowYield, Cursor, ParseResult, TokenParser},
    },
    BoaProfiler,
};

/// Break statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
/// [spec]: https://tc39.es/ecma262/#prod-BreakStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct BreakStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BreakStatement {
    /// Creates a new `BreakStatement` parser.
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

impl TokenParser for BreakStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("BreakStatement", "Parsing");
        cursor.expect(Keyword::Break, "break statement")?;

        let label = if let (true, tok) = cursor.peek_semicolon(false) {
            match tok {
                Some(tok) if tok.kind == TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = cursor.next();
                }
                _ => {}
            }

            None
        } else {
            let label = LabelIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
            cursor.expect_semicolon(false, "continue statement")?;

            Some(label)
        };

        Ok(Node::Break(label))
    }
}
