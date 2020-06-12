//! Coments lexing.

use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::Position;
use crate::syntax::lexer::{Token, TokenKind};

macro_rules! comment_match {
    () => {{
        '/'
    }};
}

/// Lexes single line comments, starting with `//`.
#[derive(Debug, Clone, Copy)]
pub(super) struct SingleLineComment;

impl<R> Tokenizer<R> for SingleLineComment {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error> {
        unimplemented!()
    }
}
