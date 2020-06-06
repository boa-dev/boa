use crate::{
    syntax::{
        ast::{node, Keyword},
        parser::{
            statement::block::Block, AllowAwait, AllowReturn, AllowYield, Cursor, ParseError,
            TokenParser,
        },
    },
    BoaProfiler,
};

/// Finally parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-Finally
#[derive(Debug, Clone, Copy)]
pub(super) struct Finally {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Finally {
    /// Creates a new `Finally` block parser.
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

impl TokenParser for Finally {
    type Output = node::Finally;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Finally", "Parsing");
        cursor.expect(Keyword::Finally, "try statement")?;
        Ok(
            Block::new(self.allow_yield, self.allow_await, self.allow_return)
                .parse(cursor)?
                .into(),
        )
    }
}
