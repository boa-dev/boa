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
use crate::Interner;
use crate::{
    syntax::{
        ast::{node::Continue, Keyword, Punctuator},
        parser::{
            cursor::{Cursor, SemicolonResult},
            statement::LabelIdentifier,
            AllowAwait, AllowYield, ParseError, TokenParser,
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
pub(super) struct ContinueStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ContinueStatement {
    /// Creates a new `ContinueStatement` parser.
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

impl<R> TokenParser<R> for ContinueStatement
where
    R: Read,
{
    type Output = Continue;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ContinueStatement", "Parsing");
        cursor.expect(Keyword::Continue, "continue statement", interner)?;

        let label = if let SemicolonResult::Found(tok) = cursor.peek_semicolon(interner)? {
            match tok {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = cursor.next(interner)?;
                }
                _ => {}
            }

            None
        } else {
            let label =
                LabelIdentifier::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;
            cursor.expect_semicolon("continue statement", interner)?;

            Some(label)
        };

        Ok(Continue::new(label))
    }
}
