//! Statement and declaration parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements
//! [spec]: https://tc39.es/ecma262/#sec-ecmascript-language-statements-and-declarations

mod block;
mod break_stm;
mod continue_stm;
mod declaration;
mod if_stm;
mod iteration;
mod return_stm;
mod switch;
mod throw;
mod try_stm;
mod variable;

use self::{
    block::BlockStatement,
    break_stm::BreakStatement,
    continue_stm::ContinueStatement,
    declaration::Declaration,
    if_stm::IfStatement,
    iteration::{DoWhileStatement, ForStatement, WhileStatement},
    return_stm::ReturnStatement,
    switch::SwitchStatement,
    throw::ThrowStatement,
    try_stm::TryStatement,
    variable::VariableStatement,
};
use super::{
    expression::Expression, AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, ParseResult,
    TokenParser,
};
use crate::syntax::ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind};

/// Statement parsing.
///
/// This can be one of the following:
///
///  - `BlockStatement`
///  - `VariableStatement`
///  - `EmptyStatement`
///  - `ExpressionStatement`
///  - `IfStatement`
///  - `BreakableStatement`
///  - `ContinueStatement`
///  - `BreakStatement`
///  - `ReturnStatement`
///  - `WithStatement`
///  - `LabelledStatement`
///  - `ThrowStatement`
///  - `TryStatement`
///  - `DebuggerStatement`
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements
/// [spec]: https://tc39.es/ecma262/#prod-Statement
#[derive(Debug, Clone, Copy)]
pub(super) struct Statement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Statement {
    /// Creates a new `Statement` parser.
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

impl TokenParser for Statement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        // TODO: add BreakableStatement and divide Whiles, fors and so on to another place.
        let tok = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;

        match tok.kind {
            TokenKind::Keyword(Keyword::If) => {
                IfStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
            }
            TokenKind::Keyword(Keyword::Var) => {
                VariableStatement::new(self.allow_yield, self.allow_await).parse(cursor)
            }
            TokenKind::Keyword(Keyword::While) => {
                WhileStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
            }
            TokenKind::Keyword(Keyword::Do) => {
                DoWhileStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
            }
            TokenKind::Keyword(Keyword::For) => {
                ForStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
            }
            TokenKind::Keyword(Keyword::Return) => {
                if self.allow_return.0 {
                    ReturnStatement::new(self.allow_yield, self.allow_await).parse(cursor)
                } else {
                    Err(ParseError::Unexpected(tok.clone(), Some("statement")))
                }
            }
            TokenKind::Keyword(Keyword::Break) => {
                BreakStatement::new(self.allow_yield, self.allow_await).parse(cursor)
            }
            TokenKind::Keyword(Keyword::Continue) => {
                ContinueStatement::new(self.allow_yield, self.allow_await).parse(cursor)
            }
            TokenKind::Keyword(Keyword::Try) => {
                TryStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
            }
            TokenKind::Keyword(Keyword::Throw) => {
                ThrowStatement::new(self.allow_yield, self.allow_await).parse(cursor)
            }
            TokenKind::Keyword(Keyword::Switch) => {
                SwitchStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                BlockStatement::new(self.allow_yield, self.allow_await, self.allow_return)
                    .parse(cursor)
            }
            // TODO: https://tc39.es/ecma262/#prod-LabelledStatement
            // TokenKind::Punctuator(Punctuator::Semicolon) => {
            //     return Ok(Node::new(NodeBase::Nope, tok.pos))
            // }
            _ => ExpressionStatement::new(self.allow_yield, self.allow_await).parse(cursor),
        }
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
pub(super) struct StatementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
    break_when_closingbrase: bool,
}

impl StatementList {
    /// Creates a new `StatementList` parser.
    pub(super) fn new<Y, A, R>(
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

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Vec<Node>, ParseError> {
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

/// Expression statement parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExpressionStatement
#[derive(Debug, Clone, Copy)]
struct ExpressionStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ExpressionStatement {
    /// Creates a new `ExpressionStatement` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl TokenParser for ExpressionStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        // TODO: lookahead
        let expr = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect_semicolon(false, "expression statement")?;

        Ok(expr)
    }
}

/// Binding identifier parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BindingIdentifier
#[derive(Debug, Clone, Copy)]
pub(super) struct BindingIdentifier {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BindingIdentifier {
    /// Creates a new `BindingIdentifier` parser.
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

impl TokenParser for BindingIdentifier {
    type Output = String;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<String, ParseError> {
        // TODO: strict mode.

        let next_token = cursor.next().ok_or(ParseError::AbruptEnd)?;

        match next_token.kind {
            TokenKind::Identifier(ref s) => Ok(s.clone()),
            TokenKind::Keyword(k @ Keyword::Yield) if !self.allow_yield.0 => Ok(k.to_string()),
            TokenKind::Keyword(k @ Keyword::Await) if !self.allow_await.0 => Ok(k.to_string()),
            _ => Err(ParseError::Expected(
                vec![TokenKind::identifier("identifier")],
                next_token.clone(),
                "binding identifier",
            )),
        }
    }
}
