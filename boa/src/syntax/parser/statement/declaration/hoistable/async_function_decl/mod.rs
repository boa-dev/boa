#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::AsyncFunctionDecl, Keyword, Punctuator},
    lexer::TokenKind,
    parser::{
        function::FormalParameters,
        function::FunctionBody,
        statement::{BindingIdentifier, LexError, Position},
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
        cursor.peek_expect_no_lineterminator(0, "async function declaration")?;
        cursor.expect(Keyword::Function, "async function declaration")?;
        let tok = cursor.peek(0)?;

        let name = if let Some(token) = tok {
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    if !self.is_default.0 {
                        return Err(ParseError::unexpected(
                            token.clone(),
                            " in async function declaration",
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

        if let Some(name) = &name {
            // Early Error: If BindingIdentifier is present and the source code matching BindingIdentifier is strict mode code,
            // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
            if cursor.strict_mode() && ["eval", "arguments"].contains(&name.as_ref()) {
                return Err(ParseError::lex(LexError::Syntax(
                    "Unexpected eval or arguments in strict mode".into(),
                    match cursor.peek(0)? {
                        Some(token) => token.span().end(),
                        None => Position::new(1, 1),
                    },
                )));
            }
        }

        let params_start_position = cursor
            .expect(Punctuator::OpenParen, "async function declaration")?
            .span()
            .end();

        let params = FormalParameters::new(false, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "async function declaration")?;
        cursor.expect(Punctuator::OpenBlock, "async function declaration")?;

        let body = FunctionBody::new(false, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "async function declaration")?;

        // Early Error: If the source code matching FormalParameters is strict mode code,
        // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
        if (cursor.strict_mode() || body.strict()) && params.has_duplicates {
            return Err(ParseError::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of AsyncFunctionBody is true
        // and IsSimpleParameterList of FormalParameters is false.
        if body.strict() && !params.is_simple {
            return Err(ParseError::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of FunctionBody.
        // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
        {
            let lexically_declared_names = body.lexically_declared_names();
            for param in params.parameters.as_ref() {
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

        Ok(AsyncFunctionDecl::new(name, params.parameters, body))
    }
}
