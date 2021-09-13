#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::declaration::generator_decl::GeneratorDecl, Keyword, Punctuator},
    parser::{
        function::FormalParameters,
        function::FunctionBody,
        statement::{BindingIdentifier, LexError, Position},
        AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use std::io::Read;

/// Generator declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function*
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct GeneratorDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl GeneratorDeclaration {
    /// Creates a new `GeneratorDeclaration` parser.
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

impl<R> TokenParser<R> for GeneratorDeclaration
where
    R: Read,
{
    type Output = GeneratorDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        cursor.expect(Keyword::Function, "generator declaration")?;
        cursor.expect(Punctuator::Mul, "generator declaration")?;

        // TODO: If self.is_default, then this can be empty.
        let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;

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

        let params_start_position = cursor
            .expect(Punctuator::OpenParen, "generator declaration")?
            .span()
            .end();

        let params = FormalParameters::new(true, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseParen, "generator declaration")?;
        cursor.expect(Punctuator::OpenBlock, "generator declaration")?;

        let body = FunctionBody::new(true, false).parse(cursor)?;

        cursor.expect(Punctuator::CloseBlock, "generator declaration")?;

        // Early Error: If the source code matching FormalParameters is strict mode code,
        // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
        if (cursor.strict_mode() || body.strict()) && params.has_duplicates {
            return Err(ParseError::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of GeneratorBody is true
        // and IsSimpleParameterList of FormalParameters is false.
        if body.strict() && !params.is_simple {
            return Err(ParseError::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of GeneratorBody.
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

        Ok(GeneratorDecl::new(name, params.parameters, body))
    }
}
