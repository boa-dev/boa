//! Continue expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
//! [spec]: https://tc39.es/ecma262/#sec-continue-statement

#[cfg(test)]
mod tests;

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::Continue, Keyword, Punctuator},
        parser::{
            cursor::{Cursor, SemicolonResult},
            statement::BindingIdentifier,
            ParseError, TokenParser,
        },
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
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
/// [spec]: https://tc39.es/ecma262/#prod-ContinueStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct ContinueStatement<const YIELD: bool, const AWAIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool> TokenParser<R> for ContinueStatement<YIELD, AWAIT>
where
    R: Read,
{
    type Output = Continue;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ContinueStatement", "Parsing");
        cursor.expect(Keyword::Continue, "continue statement")?;

        let label = if let SemicolonResult::Found(tok) = cursor.peek_semicolon()? {
            match tok {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = cursor.next();
                }
                _ => {}
            }

            None
        } else {
            let label = BindingIdentifier::<YIELD, AWAIT>.parse(cursor)?;
            cursor.expect_semicolon("continue statement")?;

            Some(label)
        };

        Ok(Continue::new::<_, Box<str>>(label))
    }
}
