//! Do-while statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
//! [spec]: https://tc39.es/ecma262/#sec-do-while-statement

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::DoWhileLoop, Keyword, Punctuator},
        parser::{
            expression::Expression, statement::Statement, AllowAwait, AllowReturn, AllowYield,
            Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};
use std::io::Read;

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

impl<R> TokenParser<R> for DoWhileStatement
where
    R: Read,
{
    type Output = DoWhileLoop;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("DoWhileStatement", "Parsing");
        cursor.expect(Keyword::Do, "do while statement")?;

        // There can be space between the Do and the body.
        cursor.skip_line_terminators()?;

        let body =
            Statement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;

        cursor.skip_line_terminators()?;

        let next_token = cursor.peek()?.ok_or(ParseError::AbruptEnd)?;

        if next_token.kind() != &TokenKind::Keyword(Keyword::While) {
            return Err(ParseError::expected(
                vec![TokenKind::Keyword(Keyword::While)],
                next_token,
                "do while statement",
            ));
        }

        cursor.skip_line_terminators()?;

        cursor.expect(Keyword::While, "do while statement")?;

        cursor.skip_line_terminators()?;

        cursor.expect(Punctuator::OpenParen, "do while statement")?;

        cursor.skip_line_terminators()?;

        let cond = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.skip_line_terminators()?;

        cursor.expect(Punctuator::CloseParen, "do while statement")?;

        expect_semicolon_dowhile(cursor)?;

        Ok(DoWhileLoop::new(body, cond))
    }
}
/// Checks that the next token is a semicolon with regards to the automatic semicolon insertion rules
/// as specified in spec.
///
/// This is used for the check at the end of a DoWhileLoop as-opposed to the regular cursor.expect() because
/// do_while represents a special condition for automatic semicolon insertion.
///
/// [spec]: https://tc39.es/ecma262/#sec-rules-of-automatic-semicolon-insertion
fn expect_semicolon_dowhile<R>(cursor: &mut Cursor<R>) -> Result<(), ParseError>
where
    R: Read,
{
    // The previous token is already known to be a CloseParan as this is checked as part of the dowhile parsing.
    // This means that a semicolon is always automatically inserted if one isn't present.

    if let Some(tk) = cursor.peek()? {
        if tk.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) {
            cursor.next()?.expect("; token vanished"); // Consume semicolon.
        }
    }

    Ok(())
}
