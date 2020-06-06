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

/// Return statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
/// [spec]: https://tc39.es/ecma262/#prod-ReturnStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct ReturnStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ReturnStatement {
    /// Creates a new `ReturnStatement` parser.
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

impl TokenParser for ReturnStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ReturnStatement", "Parsing");
        cursor.expect(Keyword::Return, "return statement")?;

        if let (true, tok) = cursor.peek_semicolon(false) {
            match tok {
                Some(tok)
                    if tok.kind == TokenKind::Punctuator(Punctuator::Semicolon)
                        || tok.kind == TokenKind::LineTerminator =>
                {
                    let _ = cursor.next();
                }
                _ => {}
            }

            return Ok(Node::Return(None));
        }

        let expr = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect_semicolon(false, "return statement")?;

        Ok(Node::return_node(expr))
    }
}
