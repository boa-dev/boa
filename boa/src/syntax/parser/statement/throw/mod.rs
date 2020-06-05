#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{Keyword, Node, Punctuator, TokenKind},
        parser::{
            expression::Expression, AllowAwait, AllowYield, Cursor, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

/// For statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw
/// [spec]: https://tc39.es/ecma262/#prod-ThrowStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct ThrowStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ThrowStatement {
    /// Creates a new `ThrowStatement` parser.
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

impl TokenParser for ThrowStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ThrowStatement", "Parsing");
        cursor.expect(Keyword::Throw, "throw statement")?;

        cursor.peek_expect_no_lineterminator(0)?;

        let expr = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
        if let Some(tok) = cursor.peek(0) {
            if tok.kind == TokenKind::Punctuator(Punctuator::Semicolon) {
                let _ = cursor.next();
            }
        }

        Ok(Node::throw(expr))
    }
}
