#[cfg(test)]
mod tests;

use super::Statement;

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::If, Keyword, Node, Punctuator},
        parser::{
            expression::Expression, AllowAwait, AllowReturn, AllowYield, Cursor, DeclaredNames,
            ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// If statement parsing.
///
/// An _If_ statement will have a condition, a block statemet, and an optional _else_ statement.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
/// [spec]: https://tc39.es/ecma262/#prod-IfStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct IfStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl IfStatement {
    /// Creates a new `IfStatement` parser.
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

impl<R> TokenParser<R> for IfStatement
where
    R: Read,
{
    type Output = If;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("IfStatement", "Parsing");
        cursor.expect(Keyword::If, "if statement")?;
        cursor.expect(Punctuator::OpenParen, "if statement")?;

        let cond = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor, env)?;

        cursor.expect(Punctuator::CloseParen, "if statement")?;

        let then_stm = Statement::new(self.allow_yield, self.allow_await, self.allow_return)
            .parse(cursor, env)?;

        let else_stm = if let Some(else_tok) = cursor.peek(0)? {
            if else_tok.kind() == &TokenKind::Keyword(Keyword::Else) {
                cursor.next()?.expect("else token vanished");
                Some(
                    Statement::new(self.allow_yield, self.allow_await, self.allow_return)
                        .parse(cursor, env)?,
                )
            } else {
                None
            }
        } else {
            None
        };

        Ok(If::new::<_, _, Node, _>(cond, then_stm, else_stm))
    }
}
