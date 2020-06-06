//! Do-while statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
//! [spec]: https://tc39.es/ecma262/#sec-do-while-statement

use crate::{
    syntax::{
        ast::{Keyword, Node, Punctuator, TokenKind},
        parser::{
            expression::Expression, statement::Statement, AllowAwait, AllowReturn, AllowYield,
            Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};

/// Do...while statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
/// [spec]: https://tc39.es/ecma262/#sec-do-while-statement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct DoWhileStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl DoWhileStatement {
    /// Creates a new `DoWhileStatement` parser.
    pub(in crate::syntax::parser::statement) fn new<Y, A, R>(
        allow_yield: Y,
        allow_await: A,
        allow_return: R,
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
        }
    }
}

impl TokenParser for DoWhileStatement {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("DoWhileStatement", "Parsing");
        cursor.expect(Keyword::Do, "do while statement")?;

        let body =
            Statement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;

        let next_token = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;

        if next_token.kind != TokenKind::Keyword(Keyword::While) {
            return Err(ParseError::expected(
                vec![TokenKind::Keyword(Keyword::While)],
                next_token.clone(),
                "do while statement",
            ));
        }

        cursor.expect(Keyword::While, "do while statement")?;
        cursor.expect(Punctuator::OpenParen, "do while statement")?;

        let cond = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "do while statement")?;
        cursor.expect_semicolon(true, "do while statement")?;

        Ok(Node::do_while_loop(body, cond))
    }
}
