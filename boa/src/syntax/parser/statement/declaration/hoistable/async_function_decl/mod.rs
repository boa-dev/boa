#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::AsyncFunctionDecl, Keyword, Punctuator},
    lexer::TokenKind,
    parser::{
        function::FormalParameters, function::FunctionBody, statement::BindingIdentifier,
        AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use std::io::Read;

/// Async Function declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/async_function
/// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#prod-AsyncFunctionDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct AsyncFunctionDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl AsyncFunctionDeclaration {
    /// Creates a new `FunctionDeclaration` parser.
    pub(super) fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_default: is_default.into(),
        }
    }
}

impl<R> TokenParser<R> for AsyncFunctionDeclaration
where
    R: Read,
{
    type Output = AsyncFunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Async, "async function declaration")?;
        cursor.expect_no_skip_lineterminator(Keyword::Function, "async function declaration")?;
        let tok = cursor.peek(0)?;

        let name = if let Some(token) = tok {
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    if !self.is_default.0 {
                        return Err(ParseError::unexpected(
                            token.clone(),
                            "Unexpected missing identifier for async function decl",
                        ));
                    }
                    None
                }
                _ => {
                    Some(BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?)
                }
            }
        } else {
            return Err(ParseError::AbruptEnd);
        };

        cursor.expect(Punctuator::OpenParen, "async function declaration")?;

        let params = FormalParameters::new(!self.allow_yield.0, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "async function declaration")?;
        cursor.expect(Punctuator::OpenBlock, "async function declaration")?;

        let body = FunctionBody::new(!self.allow_yield.0, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "async function declaration")?;

        Ok(AsyncFunctionDecl::new(name, params, body))
    }
}
