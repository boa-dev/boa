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

use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
    parser::{AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
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
        cursor.expect(Keyword::Break, "break statement")?;

        if let (true, tok) = cursor.peek_semicolon(false) {
            match tok {
                Some(tok) if tok.kind == TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = cursor.next();
                }
                _ => {}
            }

            return Ok(Node::Break(None));
        }

        let tok = cursor.next().ok_or(ParseError::AbruptEnd)?;
        // TODO: LabelIdentifier
        let node = if let TokenKind::Identifier(name) = &tok.kind {
            Node::break_node(name)
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::identifier("identifier")],
                tok.clone(),
                "break statement",
            ));
        };

        cursor.expect_semicolon(false, "break statement")?;

        Ok(node)
    }
}
