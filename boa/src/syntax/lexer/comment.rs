//! Coments lexing.

use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::Position;
use crate::syntax::lexer::Token;
use std::io::Read;

macro_rules! comment_match {
    () => {
        '/'
    };
}

/// Skips comments.
///
/// Assumes that the '/' char is already consumed.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]:
/// [mdn]:
pub(super) struct Comment;

impl Comment {
    /// Creates a new comment lexer.
    pub(super) fn new() -> Self {
        Self {}
    }
}

impl<R> Tokenizer<R> for Comment {
    fn lex(&mut self, _cursor: &mut Cursor<R>, _start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        unimplemented!()
    }
}
