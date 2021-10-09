use crate::{
    syntax::{
        ast::{node, Keyword},
        parser::{statement::block::Block, Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};

use std::io::Read;

/// Finally parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-Finally
#[derive(Debug, Clone, Copy)]
pub(super) struct Finally<const YIELD: bool, const AWAIT: bool, const RETURN: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const RETURN: bool> TokenParser<R>
    for Finally<YIELD, AWAIT, RETURN>
where
    R: Read,
{
    type Output = node::Finally;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Finally", "Parsing");
        cursor.expect(Keyword::Finally, "try statement")?;
        Ok(Block::<YIELD, AWAIT, RETURN>.parse(cursor)?.into())
    }
}
