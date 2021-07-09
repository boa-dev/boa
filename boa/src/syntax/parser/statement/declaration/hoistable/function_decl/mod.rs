#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::FunctionDecl, Keyword, Punctuator},
    parser::{
        function::FormalParameters,
        function::FunctionBody,
        statement::{BindingIdentifier, LexError, Position},
        AllowAwait, AllowDefault, AllowYield, Cursor, DeclaredNames, ParseError, TokenParser,
    },
};
use std::io::Read;

/// Function declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration

#[derive(Debug, Clone, Copy)]
pub(super) struct FunctionDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl FunctionDeclaration {
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

impl<R> TokenParser<R> for FunctionDeclaration
where
    R: Read,
{
    type Output = FunctionDecl;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        env: &mut DeclaredNames,
    ) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Function, "function declaration")?;

        // TODO: If self.is_default, then this can be empty.
        let pos = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
        let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor, env)?;

        cursor.expect(Punctuator::OpenParen, "function declaration")?;

        let params_pos = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
        let params = FormalParameters::new(false, false).parse(cursor, env)?;

        cursor.expect(Punctuator::CloseParen, "function declaration")?;
        cursor.expect(Punctuator::OpenBlock, "function declaration")?;

        let mut inner_env = DeclaredNames::default();
        for param in params.iter() {
            // TODO: Generate a better position for each parameter.
            // This can never fail, as FormalParameters makes sure that there
            // are not duplicate names.
            inner_env.insert_var_name(param.name(), params_pos).unwrap();
        }
        let body =
            FunctionBody::new(self.allow_yield, self.allow_await).parse(cursor, &mut inner_env)?;

        cursor.expect(Punctuator::CloseBlock, "function declaration")?;

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

        // Functions act like `var` statements
        env.insert_var_name(&name, pos)?;

        Ok(FunctionDecl::new(name, params, body))
    }
}
