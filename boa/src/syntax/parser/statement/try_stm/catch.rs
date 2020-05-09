use crate::syntax::{
    ast::{keyword::Keyword, node::Node, punc::Punctuator, token::TokenKind},
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
    type Output = (Node, Node);

    fn parse(self, cursor: &mut Cursor<'_>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Catch, "try statement")?;
        cursor.expect(Punctuator::OpenParen, "catch in try statement")?;
        // TODO: should accept BindingPattern
        let tok = cursor.next().ok_or(ParseError::AbruptEnd)?;
        let catch_param = if let TokenKind::Identifier(s) = &tok.kind {
            Node::local(s)
        } else {
            return Err(ParseError::Expected(
                vec![TokenKind::identifier("identifier")],
                tok.clone(),
                "catch in try statement",
            ));
        };
        cursor.expect(Punctuator::CloseParen, "catch in try statement")?;

        // Catch block
        Ok((
            Block::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?,
            catch_param,
        ))
    }
}
