//! Block statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
//! [spec]: https://tc39.es/ecma262/#sec-block

use super::{declaration::Declaration, Statement};
use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
    parser::{AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
};

/// A `BlockStatement` is equivalent to a `Block`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BlockStatement
pub(super) type BlockStatement = Block;

/// Variable declaration list parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
/// [spec]: https://tc39.es/ecma262/#prod-Block
#[derive(Debug, Clone, Copy)]
pub(super) struct Block {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Block {
    /// Creates a new `Block` parser.
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

impl TokenParser for Block {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        cursor.expect(Punctuator::OpenBlock, "block")?;
        if let Some(tk) = cursor.peek(0) {
            if tk.kind == TokenKind::Punctuator(Punctuator::CloseBlock) {
                cursor.next();
                return Ok(Node::Block(Box::new([])));
            }
        }

        let statement_list =
            StatementList::new(self.allow_yield, self.allow_await, self.allow_return, true)
                .parse(cursor)
                .map(Node::block)?;
        cursor.expect(Punctuator::CloseBlock, "block")?;

        Ok(statement_list)
    }
}

/// Reads a list of statements.
///
/// If `break_when_closingbrase` is `true`, it will stop as soon as it finds a `}` character.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-StatementList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct StatementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
    break_when_closingbrase: bool,
}

impl StatementList {
    /// Creates a new `StatementList` parser.
    pub(in crate::syntax::parser) fn new<Y, A, R>(
        allow_yield: Y,
        allow_await: A,
        allow_return: R,
        break_when_closingbrase: bool,
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
            break_when_closingbrase,
        }
    }
}

impl TokenParser for StatementList {
    type Output = Vec<Node>;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let mut items = Vec::new();

        loop {
            match cursor.peek(0) {
                Some(token) if token.kind == TokenKind::Punctuator(Punctuator::CloseBlock) => {
                    if self.break_when_closingbrase {
                        break;
                    } else {
                        return Err(ParseError::Unexpected(token.clone(), None));
                    }
                }
                None => {
                    if self.break_when_closingbrase {
                        return Err(ParseError::AbruptEnd);
                    } else {
                        break;
                    }
                }
                _ => {}
            }

            let item =
                StatementListItem::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)?;
            items.push(item);

            // move the cursor forward for any consecutive semicolon.
            while cursor.next_if(Punctuator::Semicolon).is_some() {}
        }

        Ok(items)
    }
}

/// Statement list item parsing
///
/// A statement list item can either be an statement or a declaration.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements
/// [spec]: https://tc39.es/ecma262/#prod-StatementListItem
#[derive(Debug, Clone, Copy)]
struct StatementListItem {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl StatementListItem {
    /// Creates a new `StatementListItem` parser.
    fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
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

impl TokenParser for StatementListItem {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let tok = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;

        match tok.kind {
            TokenKind::Keyword(Keyword::Function)
            | TokenKind::Keyword(Keyword::Const)
            | TokenKind::Keyword(Keyword::Let) => {
                Declaration::new(self.allow_yield, self.allow_await).parse(cursor)
            }
            _ => {
                Statement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)
            }
        }
    }
}
