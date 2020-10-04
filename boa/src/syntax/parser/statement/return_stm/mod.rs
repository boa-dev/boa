#[cfg(test)]
mod tests;

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::Return, Keyword, Node, Punctuator},
        parser::{
            cursor::{Cursor, SemicolonResult},
            expression::Expression,
            AllowAwait, AllowYield, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

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

impl<R> TokenParser<R> for ReturnStatement
where
    R: Read,
{
    type Output = Return;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ReturnStatement", "Parsing");
        cursor.expect(Keyword::Return, "return statement")?;

        if let SemicolonResult::Found(tok) = cursor.peek_semicolon()? {
            match tok {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = cursor.next();
                }
                _ => {}
            }

            return Ok(Return::new::<Node, Option<_>, Option<_>>(None, None));
        }

        let expr = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;

        cursor.expect_semicolon("return statement")?;

        Ok(Return::new(expr, None))
    }
}
