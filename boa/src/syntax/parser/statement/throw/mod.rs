#[cfg(test)]
mod tests;

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::Throw, Keyword, Punctuator},
        parser::{expression::Expression, Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};

use std::io::Read;

/// For statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw
/// [spec]: https://tc39.es/ecma262/#prod-ThrowStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct ThrowStatement<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for ThrowStatement<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Throw;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ThrowStatement", "Parsing");
        cursor.expect(Keyword::Throw, "throw statement")?;

        cursor.peek_expect_no_lineterminator(0, "throw statement")?;

        let expr = Expression::<true, YIELD, AWAIT>.parse(cursor)?;
        if let Some(tok) = cursor.peek(0)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) {
                let _ = cursor.next();
            }
        }

        Ok(Throw::new(expr))
    }
}
