//! Continue expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
//! [spec]: https://tc39.es/ecma262/#sec-continue-statement

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

/// For statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
/// [spec]: https://tc39.es/ecma262/#prod-ContinueStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct ContinueStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ContinueStatement {
    /// Creates a new `ContinueStatement` parser.
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

impl TokenParser for ContinueStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ContinueStatement", "Parsing");
        cursor.expect(Keyword::Continue, "continue statement")?;

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

        Ok(Node::Continue(label))
    }
}
