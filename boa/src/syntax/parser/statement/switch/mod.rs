#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node, node::Switch, Keyword, Node, Punctuator, TokenKind},
    parser::{
        expression::Expression, statement::StatementList, AllowAwait, AllowReturn, AllowYield,
        Cursor, ParseError, Token, TokenParser,
    },
};

/// The possible TokenKind which indicate the end of a case statement.
const CASE_BREAK_TOKENS: [TokenKind; 3] = [
    TokenKind::Punctuator(Punctuator::CloseBlock),
    TokenKind::Keyword(Keyword::Case),
    TokenKind::Keyword(Keyword::Default),
];

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
    type Output = (Box<[node::Case]>, Option<Node>);

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let mut cases = Vec::<node::Case>::new();
        let mut default: Option<Node> = None;

        cursor.expect(Punctuator::OpenBlock, "switch start case block")?;

        loop {
            match cursor.expect(Keyword::Case, "switch case: block") {
                Ok(_) => {
                    // Case statement.
                    let cond =
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

                    cursor.expect(Punctuator::Colon, "switch case block start")?;

                    let statement_list = StatementList::new(
                        self.allow_yield,
                        self.allow_await,
                        self.allow_return,
                        true,
                    )
                    .parse_generalised(cursor, Box::new(CASE_BREAK_TOKENS))?;

                    cases.push(node::Case::new(cond, statement_list));
                }
                Err(ParseError::Expected {
                    expected: _,
                    found:
                        Token {
                            kind: TokenKind::Keyword(Keyword::Default),
                            span: s,
                        },
                    context: _,
                }) => {
                    // Default statement.
                    if default.is_some() {
                        // If default has already been defined then it cannot be defined again and to do so is an error.
                        return Err(ParseError::unexpected(
                            Token::new(TokenKind::Keyword(Keyword::Default), s),
                            Some("Second default clause found in switch statement"),
                        ));
                    }

                    cursor.expect(Punctuator::Colon, "switch default case block start")?;
                    let statement_list = StatementList::new(
                        self.allow_yield,
                        self.allow_await,
                        self.allow_return,
                        true,
                    )
                    .parse_generalised(cursor, Box::new(CASE_BREAK_TOKENS))?;

                    default = Some(node::Block::from(statement_list).into());
                }
                Err(ParseError::Expected {
                    expected: _,
                    found:
                        Token {
                            kind: TokenKind::Punctuator(Punctuator::CloseBlock),
                            span: _,
                        },
                    context: _,
                }) => {
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
