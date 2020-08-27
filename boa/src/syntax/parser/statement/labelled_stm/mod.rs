use super::{LabelIdentifier, Statement};
use crate::{
    syntax::{
        ast::{node::Label, Punctuator},
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

impl TokenParser for LabelledStatement {
    type Output = Label;

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("Label", "Parsing");
        let name = LabelIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
        cursor.expect(Punctuator::Colon, "Labelled Statement")?;
        let mut stmt =
            Statement::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;
        stmt.set_label(name);
        Ok(Label::new(stmt))
    }
}
