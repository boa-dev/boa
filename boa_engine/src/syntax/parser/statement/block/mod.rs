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
use crate::syntax::{
    ast::{node, Punctuator},
    lexer::TokenKind,
    parser::{AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, TokenParser},
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use rustc_hash::{FxHashMap, FxHashSet};
use std::io::Read;

/// The possible `TokenKind` which indicate the end of a block statement.
const BLOCK_BREAK_TOKENS: [TokenKind; 1] = [TokenKind::Punctuator(Punctuator::CloseBlock)];

/// A `BlockStatement` is equivalent to a `Block`.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-BlockStatement
pub(super) type BlockStatement = Block;

/// Variable declaration list parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
/// [spec]: https://tc39.es/ecma262/#prod-Block
#[derive(Debug, Clone, Copy)]
pub(super) struct Block {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Block {
    /// Creates a new `Block` parser.
    pub(super) fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl<R> TokenParser<R> for Block
where
    R: Read,
{
    type Output = node::Block;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("Block", "Parsing");
        cursor.expect(Punctuator::OpenBlock, "block", interner)?;
        if let Some(tk) = cursor.peek(0, interner)? {
            if tk.kind() == &TokenKind::Punctuator(Punctuator::CloseBlock) {
                cursor.next(interner)?.expect("} token vanished");
                return Ok(node::Block::from(vec![]));
            }
        }
        let position = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .span()
            .start();
        let statement_list = StatementList::new(
            self.allow_yield,
            self.allow_await,
            self.allow_return,
            &BLOCK_BREAK_TOKENS,
        )
        .parse(cursor, interner)
        .map(node::Block::from)?;
        cursor.expect(Punctuator::CloseBlock, "block", interner)?;

        let lexically_declared_names = statement_list.lexically_declared_names();
        let mut lexically_declared_names_map: FxHashMap<Sym, bool> = FxHashMap::default();
        for (name, is_function_declaration) in &lexically_declared_names {
            if let Some(existing_is_function_declaration) = lexically_declared_names_map.get(name) {
                if !(!cursor.strict_mode()
                    && *is_function_declaration
                    && *existing_is_function_declaration)
                {
                    return Err(ParseError::general(
                        "lexical name declared multiple times",
                        position,
                    ));
                }
            }
            lexically_declared_names_map.insert(*name, *is_function_declaration);
        }

        let mut var_declared_names = FxHashSet::default();
        for node in statement_list.items() {
            node.var_declared_names(&mut var_declared_names);
        }
        for (lex_name, _) in &lexically_declared_names {
            if var_declared_names.contains(lex_name) {
                return Err(ParseError::general(
                    "lexical name declared in var names",
                    position,
                ));
            }
        }

        Ok(statement_list)
    }
}
