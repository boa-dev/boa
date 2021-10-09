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

use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{Keyword, Node},
        parser::{Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};

use std::io::Read;

/// Parses a declaration.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Declaration
#[derive(Debug, Clone, Copy)]
pub(super) struct Declaration<const YIELD: bool, const AWAIT: bool, const CONST_INIT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const CONST_INIT: bool> TokenParser<R>
    for Declaration<YIELD, AWAIT, CONST_INIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Declaration", "Parsing");
        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function) | TokenKind::Keyword(Keyword::Async) => {
                HoistableDeclaration::<YIELD, AWAIT, false>.parse(cursor)
            }
            TokenKind::Keyword(Keyword::Const) | TokenKind::Keyword(Keyword::Let) => {
                LexicalDeclaration::<true, YIELD, AWAIT, CONST_INIT>.parse(cursor)
            }
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}
