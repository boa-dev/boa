//! Declaration parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements#Declarations
//! [spec]:https://tc39.es/ecma262/#sec-declarations-and-the-variable-statement

pub(in crate::syntax::parser) mod hoistable;
mod lexical;
#[cfg(test)]
mod tests;

use self::{hoistable::HoistableDeclaration, lexical::LexicalDeclaration};
use crate::syntax::{
    ast::{Keyword, Node},
    lexer::TokenKind,
    parser::{AllowAwait, AllowYield, Cursor, ParseError, TokenParser},
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Parses a declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Declaration
#[derive(Debug, Clone, Copy)]
pub(super) struct Declaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    const_init_required: bool,
}

impl Declaration {
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, const_init_required: bool) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            const_init_required,
        }
    }
}

impl<R> TokenParser<R> for Declaration
where
    R: Read,
{
    type Output = Node;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("Declaration", "Parsing");
        let tok = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function | Keyword::Async) => {
                HoistableDeclaration::new(self.allow_yield, self.allow_await, false)
                    .parse(cursor, interner)
            }
            TokenKind::Keyword(Keyword::Const | Keyword::Let) => LexicalDeclaration::new(
                true,
                self.allow_yield,
                self.allow_await,
                self.const_init_required,
            )
            .parse(cursor, interner),
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}
