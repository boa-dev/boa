//! Break expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
//! [spec]: https://tc39.es/ecma262/#sec-break-statement

#[cfg(test)]
mod tests;

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::Break, Keyword, Punctuator},
        parser::{
            cursor::{Cursor, SemicolonResult},
            ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

use super::BindingIdentifier;

/// Break statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
/// [spec]: https://tc39.es/ecma262/#prod-BreakStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct BreakStatement<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for BreakStatement<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Break;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("BreakStatement", "Parsing");
        cursor.expect(Keyword::Break, "break statement")?;

        let label = if let SemicolonResult::Found(tok) = cursor.peek_semicolon()? {
            match tok {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = cursor.next()?;
                }
                _ => {}
            }

            None
        } else {
            let label = BindingIdentifier::<YIELD, AWAIT>.parse(cursor)?;
            cursor.expect_semicolon("break statement")?;

            Some(label)
        };

        Ok(Break::new::<_, Box<str>>(label))
    }
}
