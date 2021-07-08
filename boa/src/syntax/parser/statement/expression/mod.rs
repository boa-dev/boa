use super::super::{expression::Expression, ParseResult};
use crate::{
    syntax::{
        ast::node::Node,
        parser::{AllowAwait, AllowYield, Cursor, DeclaredNames, TokenParser},
    },
    BoaProfiler,
};
use std::io::Read;

/// Expression statement parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ExpressionStatement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser::statement) struct ExpressionStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ExpressionStatement {
    /// Creates a new `ExpressionStatement` parser.
    pub(in crate::syntax::parser::statement) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ExpressionStatement
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, env: &mut DeclaredNames) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("ExpressionStatement", "Parsing");
        // TODO: lookahead
        let expr = Expression::new(true, self.allow_yield, self.allow_await).parse(cursor, env)?;

        cursor.expect_semicolon("expression statement")?;

        Ok(expr)
    }
}
