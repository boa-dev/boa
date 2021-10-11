#[cfg(test)]
mod tests;

use crate::{
    syntax::{
        ast::{node::AsyncFunctionExpr, Keyword, Punctuator},
        lexer::{Error as LexError, Position, TokenKind},
        parser::{
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};

use std::io::Read;

/// Async Function expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/async_function
/// [spec]: https://tc39.es/ecma262/#prod-AsyncFunctionExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct AsyncFunctionExpression {
    allow_yield: AllowYield,
}

impl AsyncFunctionExpression {
    /// Creates a new `AsyncFunctionExpression` parser.
    pub(super) fn new<Y>(allow_yield: Y) -> Self
    where
        Y: Into<AllowYield>,
    {
        Self {
            allow_yield: allow_yield.into(),
        }
    }
}

impl<R> TokenParser<R> for AsyncFunctionExpression
where
    R: Read,
{
    type Output = AsyncFunctionExpr;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("AsyncFunctionExpression", "Parsing");

        let tok = cursor.peek(0)?;

        let name = if let Some(token) = tok {
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => None,
                _ => Some(BindingIdentifier::new(self.allow_yield, true).parse(cursor)?),
            }
        } else {
            return Err(ParseError::AbruptEnd);
        };

        // Early Error: If BindingIdentifier is present and the source code matching BindingIdentifier is strict mode code,
        // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
        if let Some(name) = &name {
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
            .expect(Punctuator::OpenParen, "async function expression")?
            .span()
            .end();

        let params = FormalParameters::new(false, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "async function expression")?;
        cursor.expect(Punctuator::OpenBlock, "async function expression")?;

        let body = FunctionBody::new(false, true).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "async function expression")?;

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

        Ok(AsyncFunctionExpr::new(name, params.parameters, body))
    }
}
