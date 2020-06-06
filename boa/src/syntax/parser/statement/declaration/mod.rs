//! Declaration parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements#Declarations
//! [spec]:https://tc39.es/ecma262/#sec-declarations-and-the-variable-statement

mod hoistable;
mod lexical;
#[cfg(test)]
mod tests;

use self::{hoistable::HoistableDeclaration, lexical::LexicalDeclaration};
use crate::{
    syntax::{
        ast::{Keyword, Node, TokenKind},
        parser::{AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
    },
    BoaProfiler,
};

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
}

impl Declaration {
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

impl TokenParser for Declaration {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("Declaration", "Parsing");
        let tok = cursor.peek(0).ok_or(ParseError::AbruptEnd)?;

        match tok.kind {
            TokenKind::Keyword(Keyword::Function) => {
                HoistableDeclaration::new(self.allow_yield, self.allow_await, false).parse(cursor)
            }
            TokenKind::Keyword(Keyword::Const) | TokenKind::Keyword(Keyword::Let) => {
                LexicalDeclaration::new(true, self.allow_yield, self.allow_await).parse(cursor)
            }
            _ => unreachable!("unknown token found"),
        }
    }
}
