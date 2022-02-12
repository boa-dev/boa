#[cfg(test)]
mod tests;

use crate::syntax::lexer::TokenKind;
use crate::Interner;
use crate::{
    syntax::{
        ast::{node::Throw, Keyword, Punctuator},
        parser::{expression::Expression, AllowAwait, AllowYield, Cursor, ParseError, TokenParser},
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
pub(super) struct ThrowStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ThrowStatement {
    /// Creates a new `ThrowStatement` parser.
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

impl<R> TokenParser<R> for ThrowStatement
where
    R: Read,
{
    type Output = Throw;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ThrowStatement", "Parsing");
        cursor.expect(Keyword::Throw, "throw statement", interner)?;

        cursor.peek_expect_no_lineterminator(0, "throw statement", interner)?;

        let expr =
            Expression::new(true, self.allow_yield, self.allow_await).parse(cursor, interner)?;
        if let Some(tok) = cursor.peek(0, interner)? {
            if tok.kind() == &TokenKind::Punctuator(Punctuator::Semicolon) {
                let _next = cursor.next(interner).expect("token disappeared");
            }
        }

        Ok(Throw::new(expr))
    }
}
