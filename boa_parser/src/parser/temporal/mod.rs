//! Parsers for `Temporal` syntax.

#![allow(unused_variables)] // Unimplemented

use crate::ParseResult;

use super::{cursor::Cursor, TokenParser};
use boa_ast::UtcOffset;
use boa_interner::Interner;
use std::io::Read;

/// `TimeZoneNumericUTCOffset` parser.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/proposal-temporal/#prod-TimeZoneNumericUTCOffset
#[derive(Debug, Clone, Copy)]
pub struct TimeZoneNumericUTCOffset;

impl<R> TokenParser<R> for TimeZoneNumericUTCOffset
where
    R: Read,
{
    type Output = UtcOffset;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        todo!()
    }
}
