//! Coments lexing.

use super::{Cursor, Error, Tokenizer};
use crate::syntax::ast::bigint::BigInt;
use crate::syntax::ast::{
    token::{NumericLiteral, Token, TokenKind},
    Position, Punctuator, Span,
};
use std::{
    char::{decode_utf16, from_u32},
    fmt,
    io::{self, BufRead, Bytes, Read, Seek},
    iter::Peekable,
    str::{Chars, FromStr},
};

/// Lexes single line comments, starting with `//`.
#[derive(Debug, Clone, Copy)]
pub(super) struct SingleLineComment;

impl<R> Tokenizer<R> for SingleLineComment {
    fn lex(&mut self, cursor: &mut Cursor<R>, start_pos: Position) -> Result<Token, Error> {
        unimplemented!()
    }
}
