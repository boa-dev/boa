use crate::syntax::{
    ast::{node::Node, token::TokenKind},
    parser::{AllowAwait, AllowYield, Cursor, ParseError, ParseResult, TokenParser},
};

/// CatchParameter parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#prod-CatchParameter
#[derive(Debug, Clone, Copy)]
pub(super) struct CatchParameter {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl CatchParameter {
    /// Creates a new `CatchParameter` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl TokenParser for CatchParameter {
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<'_>) -> ParseResult {
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
        Ok(catch_param)
    }
}
