#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::AsyncFunctionDecl, Keyword, Punctuator},
    lexer::TokenKind,
    parser::{
        function::FormalParameters,
        function::FunctionBody,
        statement::{BindingIdentifier, LexError, Position},
        AllowAwait, AllowDefault, AllowYield, Cursor, DeclaredNames, ParseError, TokenParser,
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Async, "async function declaration")?;
        cursor.peek_expect_no_lineterminator(0, "async function declaration")?;
        cursor.expect(Keyword::Function, "async function declaration")?;
        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        let pos = tok.span().start();
        let name = match tok.kind() {
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                if !self.is_default.0 {
                    return Err(ParseError::unexpected(
                        tok.clone(),
                        " in async function declaration",
                    ));
                }
                None
            }
            _ => Some(
                BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor, env)?,
            ),
        };

        cursor.expect(Punctuator::OpenParen, "async function declaration")?;

        let params = FormalParameters::new(false, true).parse(cursor, env)?;

        cursor.expect(Punctuator::CloseParen, "async function declaration")?;
        cursor.expect(Punctuator::OpenBlock, "async function declaration")?;

        let mut inner_env = DeclaredNames::default();
        for param in params.iter() {
            // This can never fail, as FormalParameters makes sure that there
            // are not duplicate names.
            inner_env
                .insert_var_name(param.name(), Position::new(1, 1))
                .unwrap();
        }
        let body = FunctionBody::new(false, true).parse(cursor, &mut inner_env)?;

        cursor.expect(Punctuator::CloseBlock, "async function declaration")?;

        // It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of FunctionBody.
        // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
        {
            let lexically_declared_names = body.lexically_declared_names();
            for param in params.as_ref() {
                if lexically_declared_names.contains(param.name()) {
                    return Err(ParseError::lex(LexError::Syntax(
                        format!("Redeclaration of formal parameter `{}`", param.name()).into(),
                        match cursor.peek(0)? {
                            Some(token) => token.span().end(),
                            None => Position::new(1, 1),
                        },
                    )));
                }
            }
        }

        if let Some(ref name) = name {
            env.insert_func_name(name, pos)?;
        }

        Ok(AsyncFunctionDecl::new(name, params, body))
    }
}
