use std::io::Read;

use super::{LabelIdentifier, Statement};
use crate::{
    syntax::{
        ast::{Node, NodeKind, Punctuator},
        parser::{
            cursor::Cursor, error::ParseError, AllowAwait, AllowReturn, AllowYield, TokenParser,
        },
    },
    BoaProfiler,
};
/// Labelled Statement Parsing
///
/// More information
/// - [MDN documentation][mdn]
/// - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/label
/// [spec]: https://tc39.es/ecma262/#sec-labelled-statements
#[derive(Debug, Clone, Copy)]
pub(super) struct LabelledStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl LabelledStatement {
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

impl<R> TokenParser<R> for LabelledStatement
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Label", "Parsing");

        let (name, name_span) =
            LabelIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
        cursor.expect(Punctuator::Colon, "Labelled Statement")?;
        let mut stmt =
            Statement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;

        set_label_for_node(&mut stmt, name);
        Ok(stmt)
    }
}

fn set_label_for_node(stmt: &mut Node, name: Box<str>) {
    match stmt.kind() {
        NodeKind::ForLoop(ref mut for_loop) => for_loop.set_label(name),
        NodeKind::ForOfLoop(ref mut for_of_loop) => for_of_loop.set_label(name),
        NodeKind::ForInLoop(ref mut for_in_loop) => for_in_loop.set_label(name),
        NodeKind::DoWhileLoop(ref mut do_while_loop) => do_while_loop.set_label(name),
        NodeKind::WhileLoop(ref mut while_loop) => while_loop.set_label(name),
        _ => (),
    }
}
