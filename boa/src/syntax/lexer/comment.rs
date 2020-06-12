//! Coments lexing.

use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::Position;
use crate::syntax::lexer::{Token, TokenKind};
use std::{
    char::{decode_utf16, from_u32},
    convert::TryFrom,
    io::{self, ErrorKind, Read},
    str,
};

macro_rules! comment_match {
    () => {{
        '/'
    }};
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
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error>
    where
        R: Read,
    {
        unimplemented!()
    }
}

