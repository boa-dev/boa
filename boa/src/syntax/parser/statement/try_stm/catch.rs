use super::catchparam::CatchParameter;
use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator},
    parser::{
        statement::block::Block, AllowAwait, AllowReturn, AllowYield, Cursor, ParseError,
        TokenParser,
    },
};

/// Catch parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-Catch
#[derive(Debug, Clone, Copy)]
pub(super) struct Catch {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl Catch {
    /// Creates a new `Catch` block parser.
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

impl TokenParser for Catch {
    type Output = (Option<Node>, Option<Node>);

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Catch, "try statement")?;
        let catch_param = if cursor.next_if(Punctuator::OpenParen).is_some() {
            let catch_param =
                CatchParameter::new(self.allow_yield, self.allow_await).parse(cursor)?;
            cursor.expect(Punctuator::CloseParen, "catch in try statement")?;
            Some(catch_param)
        } else {
            None
        };

        // Catch block
        Ok((
            Some(Block::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?),
            catch_param,
        ))
    }
}
