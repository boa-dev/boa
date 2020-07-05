use super::LabelIdentifier;
use crate::{
    syntax::{
        ast::Punctuator,
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
pub(super) struct Label {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Label {
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

impl TokenParser for Label {
    type Output = String;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Label", "Parsing");
        let name = LabelIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
        cursor.expect(Punctuator::Colon, "Labelled Statement")?;
        Ok(name.to_string())
    }
}
