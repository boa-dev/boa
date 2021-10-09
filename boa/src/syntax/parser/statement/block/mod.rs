//! Block statement parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
//! [spec]: https://tc39.es/ecma262/#sec-block

#[cfg(test)]
mod tests;

use super::StatementList;

use crate::syntax::lexer::TokenKind;
use crate::{
    profiler::BoaProfiler,
    syntax::{
        ast::{node, Punctuator},
        parser::{Cursor, ParseError, TokenParser},
    },
};

use std::io::Read;

/// The possible TokenKind which indicate the end of a block statement.
const BLOCK_BREAK_TOKENS: [TokenKind; 1] = [TokenKind::Punctuator(Punctuator::CloseBlock)];

/// Variable declaration list parsing.
///
/// A `Block` is equivalent to a `BlockStatement`.
///
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec_block]
/// - [ECMAScript specification][spec_statement]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
/// [spec_block]: https://tc39.es/ecma262/#prod-Block
/// [spec_statement]: https://tc39.es/ecma262/#prod-BlockStatement
#[derive(Debug, Clone, Copy)]
pub(super) struct Block<const YIELD: bool, const AWAIT: bool, const RETURN: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for Block<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = node::Block;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Block", "Parsing");
        cursor.expect(Punctuator::OpenBlock, "block")?;
        if let Some(tk) = cursor.peek(0)? {
            if tk.kind() == &TokenKind::Punctuator(Punctuator::CloseBlock) {
                cursor.next()?.expect("} token vanished");
                return Ok(node::Block::from(vec![]));
            }
        }

        let statement_list = StatementList::<YIELD, AWAIT, RETURN, true>::new(&BLOCK_BREAK_TOKENS)
            .parse(cursor)
            .map(node::Block::from)?;
        cursor.expect(Punctuator::CloseBlock, "block")?;

        Ok(statement_list)
    }
}
