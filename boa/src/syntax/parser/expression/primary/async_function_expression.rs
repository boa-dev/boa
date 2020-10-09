//! Async Function expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]:
//! [spec]:

use crate::{
    syntax::{
        ast::node::AsyncFunctionExpr,
        parser::{Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};

use std::io::Read;

/// Async Function expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]:
/// [spec]:
#[derive(Debug, Clone, Copy)]
pub(super) struct AsyncFunctionExpression;

impl<R> TokenParser<R> for AsyncFunctionExpression
where
    R: Read,
{
    type Output = AsyncFunctionExpr;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("AsyncFunctionExpression", "Parsing");

        unimplemented!("Async function expression parse");
    }
}
