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

use crate::{
    lexer::TokenKind,
    parser::{
        statement::StatementList, AllowAwait, AllowReturn, AllowYield, Cursor, OrAbrupt,
        ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    operations::{lexically_declared_names_legacy, var_declared_names},
    statement, Punctuator,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use rustc_hash::FxHashMap;
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
    type Output = statement::Block;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("Block", "Parsing");
        cursor.expect(Punctuator::OpenBlock, "block", interner)?;
        if let Some(tk) = cursor.peek(0, interner)? {
            if tk.kind() == &TokenKind::Punctuator(Punctuator::CloseBlock) {
                cursor.advance(interner);
                return Ok(statement::Block::from(vec![]));
            }
        }
        let position = cursor.peek(0, interner).or_abrupt()?.span().start();
        let statement_list = StatementList::new(
            self.allow_yield,
            self.allow_await,
            self.allow_return,
            &BLOCK_BREAK_TOKENS,
        )
        .parse(cursor, interner)
        .map(statement::Block::from)?;
        cursor.expect(Punctuator::CloseBlock, "block", interner)?;

        // It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate
        // entries, unless the source text matched by this production is not strict mode code and the
        // duplicate entries are only bound by FunctionDeclarations.
        let mut lexical_names = FxHashMap::default();
        for (name, is_fn) in lexically_declared_names_legacy(&statement_list) {
            if let Some(is_fn_previous) = lexical_names.insert(name, is_fn) {
                match (cursor.strict_mode(), is_fn, is_fn_previous) {
                    (false, true, true) => {}
                    _ => {
                        return Err(Error::general(
                            "lexical name declared multiple times",
                            position,
                        ));
                    }
                }
            }
        }

        // It is a Syntax Error if any element of the LexicallyDeclaredNames of StatementList also
        // occurs in the VarDeclaredNames of StatementList.
        for name in var_declared_names(&statement_list) {
            if lexical_names.contains_key(&name) {
                return Err(Error::general(
                    "lexical name declared in var names",
                    position,
                ));
            }
        }

        Ok(statement_list)
    }
}
