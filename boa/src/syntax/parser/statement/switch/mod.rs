#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{
        node,
        node::Switch,
        Keyword, Node, Punctuator,
    },
    parser::{
        expression::Expression, AllowAwait, AllowReturn, AllowYield, Cursor, ParseError,
        TokenParser, Token, statement::Statement, TokenKind,
    },
};

/// Switch statement parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
/// [spec]: https://tc39.es/ecma262/#prod-SwitchStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct SwitchStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl SwitchStatement {
    /// Creates a new `SwitchStatement` parser.
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

impl TokenParser for SwitchStatement {
    type Output = Switch;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Switch, "switch statement")?;
        cursor.expect(Punctuator::OpenParen, "switch statement")?;

        let condition = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "switch statement")?;

        let (cases, default) =
            CaseBlock::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;

        Ok(Switch::new(condition, cases, default))
    }
}

/// Switch case block parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CaseBlock
#[derive(Debug, Clone, Copy)]
struct CaseBlock {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl CaseBlock {
    /// Creates a new `CaseBlock` parser.
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

impl TokenParser for CaseBlock {
    type Output = (Box<[node::Case]>, Option<node::Case>);

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let cases = Vec::<node::Case>::new();
        let default: Option<node::Case>;

        cursor.expect(Punctuator::OpenBlock, "switch start case block")?;

        loop {
            match cursor.expect(Keyword::Case, "switch case: block"){
                Ok(_) => {
                    // Case statement list.
                    cursor.expect(Punctuator::Colon, "switch case block start")?;
                    cases.push(CaseStatement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?);
                }
                Err(ParseError::Expected{expected: _, found: Token{kind: TokenKind::Keyword(Keyword::Default), span: _}, context: _}) => {
                     // Default statement list.
                    cursor.expect(Punctuator::Colon, "switch case block start")?;
                    default = Some(CaseStatement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?);
                }
                Err(ParseError::Expected{expected: _, found: Token{kind: TokenKind::Punctuator(Punctuator::CloseBlock), span: _}, context: _}) => {
                    // End of switch block.
                    break;
                }
                Err(e) => {
                    // Unexpected statement.
                    return Err(e);
                }
            }
        }

        Ok((cases.into_boxed_slice(), default))
    }
}

/// A list of statements within a switch case/default block.
#[derive(Debug, Clone, Copy)]
struct CaseStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl CaseStatement {
    /// Creates a new case statement parser.
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

impl TokenParser for CaseStatement {
    type Output = node::Case;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        
    }
}