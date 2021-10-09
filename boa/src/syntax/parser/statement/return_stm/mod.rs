#[cfg(test)]
mod tests;

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::Return, Keyword, Node, Punctuator},
        parser::{
            cursor::{Cursor, SemicolonResult},
            expression::Expression,
            ParseError, TokenParser,
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
pub(super) struct ReturnStatement<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for ReturnStatement<YIELD, AWAIT>
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

        let expr = Expression::<true, YIELD, AWAIT>.parse(cursor)?;

        cursor.expect_semicolon("return statement")?;

        Ok(Return::new(expr, None))
    }
}
