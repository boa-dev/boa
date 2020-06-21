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
            statement::LabelIdentifier, AllowAwait, AllowYield, ParseError, Parser, TokenParser,
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

    fn parse(self, parser: &mut Parser<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ContinueStatement", "Parsing");
        parser.expect(Keyword::Continue, "continue statement")?;

        let label = if let (true, tok) = parser.peek_semicolon(false) {
            match tok {
                Some(tok) if tok.kind == TokenKind::Punctuator(Punctuator::Semicolon) => {
                    let _ = parser.next();
                }
                _ => {}
            }

            None
        } else {
            let label = LabelIdentifier::new(self.allow_yield, self.allow_await).parse(parser)?;
            parser.expect_semicolon(false, "continue statement")?;

            Some(label)
        };

        Ok(Continue::new::<_, Box<str>>(label))
    }
}
