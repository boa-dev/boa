use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::{Position, Span};
use crate::syntax::lexer::{Token, TokenKind};
use std::{
    char::{decode_utf16, from_u32},
    convert::TryFrom,
    io::{self, ErrorKind, Read},
    str,
};

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
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        unimplemented!()
    }
}
