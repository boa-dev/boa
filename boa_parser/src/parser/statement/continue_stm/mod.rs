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

use crate::{
    lexer::TokenKind,
    parser::{
        cursor::{Cursor, SemicolonResult},
        expression::LabelIdentifier,
        AllowAwait, AllowYield, ParseResult, TokenParser,
    },
};
use boa_ast::{statement::Continue, Keyword, Punctuator};
use boa_interner::Interner;
use boa_profiler::Profiler;
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

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ContinueStatement", "Parsing");
        cursor.expect((Keyword::Continue, false), "continue statement", interner)?;

        let label = if let SemicolonResult::Found(tok) = cursor.peek_semicolon(interner)? {
            if let Some(token) = tok {
                if token.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) {
                    cursor.advance(interner);
                } else if token.kind() == &TokenKind::LineTerminator {
                    if let Some(token) = cursor.peek(0, interner)? {
                        if token.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) {
                            cursor.advance(interner);
                        }
                    }
                }
            }

            None
        } else {
            let label = LabelIdentifier::new(self.allow_yield, self.allow_await)
                .parse(cursor, interner)?
                .sym();
            cursor.expect_semicolon("continue statement", interner)?;

            Some(label)
        };

        Ok(Continue::new(label))
    }
}
