#[cfg(test)]
mod tests;

use crate::syntax::{
    ast::{node::AsyncFunctionExpr, Keyword, Position, Punctuator},
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::BindingIdentifier,
        function::{FormalParameters, FunctionBody},
        AllowYield, Cursor, ParseError, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
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
    name: Option<Sym>,
    allow_yield: AllowYield,
}

impl AsyncFunctionExpression {
    /// Creates a new `AsyncFunctionExpression` parser.
    pub(super) fn new<N, Y>(name: N, allow_yield: Y) -> Self
    where
        N: Into<Option<Sym>>,
        Y: Into<AllowYield>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
        }
    }
}

impl<R> TokenParser<R> for AsyncFunctionExpression
where
    R: Read,
{
    type Output = AsyncFunctionExpr;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("AsyncFunctionExpression", "Parsing");
        cursor.peek_expect_no_lineterminator(0, "async function expression", interner)?;
        cursor.expect(
            (Keyword::Function, false),
            "async function expression",
            interner,
        )?;

        let name = match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            TokenKind::Punctuator(Punctuator::OpenParen) => self.name,
            _ => Some(BindingIdentifier::new(self.allow_yield, true).parse(cursor, interner)?),
        };

        // Early Error: If BindingIdentifier is present and the source code matching BindingIdentifier is strict mode code,
        // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
        if let Some(name) = name {
            if cursor.strict_mode() && [Sym::EVAL, Sym::ARGUMENTS].contains(&name) {
                return Err(ParseError::lex(LexError::Syntax(
                    "Unexpected eval or arguments in strict mode".into(),
                    match cursor.peek(0, interner)? {
                        Some(token) => token.span().end(),
                        None => Position::new(1, 1),
                    },
                )));
            }
        }

        let params_start_position = cursor
            .expect(Punctuator::OpenParen, "async function expression", interner)?
            .span()
            .end();

        let params = FormalParameters::new(false, true).parse(cursor, interner)?;

        cursor.expect(
            Punctuator::CloseParen,
            "async function expression",
            interner,
        )?;
        cursor.expect(Punctuator::OpenBlock, "async function expression", interner)?;

        let body = FunctionBody::new(false, true).parse(cursor, interner)?;

        cursor.expect(
            Punctuator::CloseBlock,
            "async function expression",
            interner,
        )?;

        // Early Error: If the source code matching FormalParameters is strict mode code,
        // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
        if (cursor.strict_mode() || body.strict()) && params.has_duplicates() {
            return Err(ParseError::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of AsyncFunctionBody is true
        // and IsSimpleParameterList of FormalParameters is false.
        if body.strict() && !params.is_simple() {
            return Err(ParseError::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of FunctionBody.
        // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
        params.name_in_lexically_declared_names(
            &body.lexically_declared_names_top_level(),
            params_start_position,
        )?;

        Ok(AsyncFunctionExpr::new(name, params, body))
    }
}
