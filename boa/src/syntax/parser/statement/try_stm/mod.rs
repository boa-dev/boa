#[cfg(test)]
mod tests;

use super::block::Block;
use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
    parser::{
        statement::BindingIdentifier, AllowAwait, AllowReturn, AllowYield, Cursor, ParseError,
        ParseResult, TokenParser,
    },
};

/// Try...catch statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#sec-try-statement
#[derive(Debug, Clone, Copy)]
pub(super) struct TryStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl TryStatement {
    /// Creates a new `TryStatement` parser.
    pub(super) fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl TokenParser for TryStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        // TRY
        cursor.expect(Keyword::Try, "try statement")?;

        let try_clause =
            Block::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;

        let next_token = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;

        if next_token.kind != TokenKind::Keyword(Keyword::Catch)
            && next_token.kind != TokenKind::Keyword(Keyword::Finally)
        {
            return Err(ParseError::Expected(
                vec![
                    TokenKind::Keyword(Keyword::Catch),
                    TokenKind::Keyword(Keyword::Finally),
                ],
                next_token.clone(),
                "try statement",
            ));
        }

        // CATCH
        let (catch, param) = if next_token.kind == TokenKind::Keyword(Keyword::Catch) {
            // Catch binding
            cursor.expect(Punctuator::OpenParen, "catch in try statement")?;
            // TODO: CatchParameter - BindingPattern
            let catch_param = BindingIdentifier::new(self.allow_yield, self.allow_await)
                .parse(cursor)
                .map(Node::local)?;
            cursor.expect(Punctuator::CloseParen, "catch in try statement")?;

            // Catch block
            (
                Some(
                    Block::new(self.allow_yield, self.allow_await, self.allow_return)
                        .parse(cursor)?,
                ),
                Some(catch_param),
            )
        } else {
            (None, None)
        };

        // FINALLY
        let finally_block = if cursor.next_if(Keyword::Finally).is_some() {
            Some(Block::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?)
        } else {
            None
        };

        Ok(Node::try_node::<_, _, _, _, Node, Node, Node>(
            try_clause,
            catch,
            param,
            finally_block,
        ))
    }
}
