use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::Position;
use crate::syntax::lexer::Token;
use std::io::Read;

#[derive(Debug, Clone, Copy)]
pub(super) struct Operator {
    init: char,
}

impl Operator {
    /// Creates a new string literal lexer.
    pub(super) fn new(init: char) -> Self {
        Self { init }
    }
}

impl<R> Tokenizer<R> for Operator {
    fn lex(&mut self, _cursor: &mut Cursor<R>, _start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        unimplemented!()
    }
}
