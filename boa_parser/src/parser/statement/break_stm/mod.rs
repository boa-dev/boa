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

use crate::{
    lexer::TokenKind,
    parser::{
        cursor::{Cursor, SemicolonResult},
        expression::LabelIdentifier,
        AllowAwait, AllowYield, ParseResult, TokenParser,
    },
};
use boa_ast::{statement::Break, Keyword, Punctuator};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Break statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
/// [spec]: https://tc39.es/ecma262/#prod-BreakStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct BreakStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl BreakStatement {
    /// Creates a new `BreakStatement` parser.
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

impl<R> TokenParser<R> for BreakStatement
where
    R: Read,
{
    type Output = Break;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("BreakStatement", "Parsing");
        cursor.expect((Keyword::Break, false), "break statement", interner)?;

        let label = if let SemicolonResult::Found(tok) = cursor.peek_semicolon(interner)? {
            match tok {
                Some(tok) if tok.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) => {
                    cursor.advance(interner);
                }
                _ => {}
            }

            None
        } else {
            let label = LabelIdentifier::new(self.allow_yield, self.allow_await)
                .parse(cursor, interner)?
                .sym();
            cursor.expect_semicolon("break statement", interner)?;

            Some(label)
        };

        Ok(Break::new(label))
    }
}
