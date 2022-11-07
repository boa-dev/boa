//! Generator expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function*
//! [spec]: https://tc39.es/ecma262/#prod-GeneratorExpression

#[cfg(test)]
mod tests;

use crate::{
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::BindingIdentifier,
        function::{FormalParameters, FunctionBody},
        name_in_lexically_declared_names, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    expression::Identifier,
    function::Generator,
    operations::{bound_names, contains, top_level_lexically_declared_names, ContainsSymbol},
    Position, Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Generator expression parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function*
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct GeneratorExpression {
    name: Option<Identifier>,
}

impl GeneratorExpression {
    /// Creates a new `GeneratorExpression` parser.
    pub(in crate::parser) fn new<N>(name: N) -> Self
    where
        N: Into<Option<Identifier>>,
    {
        Self { name: name.into() }
    }
}

impl<R> TokenParser<R> for GeneratorExpression
where
    R: Read,
{
    type Output = Generator;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("GeneratorExpression", "Parsing");

        cursor.expect(
            TokenKind::Punctuator(Punctuator::Mul),
            "generator expression",
            interner,
        )?;

        let (name, has_binding_identifier) = match cursor.peek(0, interner).or_abrupt()?.kind() {
            TokenKind::Punctuator(Punctuator::OpenParen) => (self.name, false),
            _ => (
                Some(BindingIdentifier::new(true, false).parse(cursor, interner)?),
                true,
            ),
        };

        // If BindingIdentifier is present and the source text matched by BindingIdentifier is strict mode code,
        // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
        // https://tc39.es/ecma262/#sec-generator-function-definitions-static-semantics-early-errors
        if let Some(name) = name {
            if cursor.strict_mode() && [Sym::EVAL, Sym::ARGUMENTS].contains(&name.sym()) {
                return Err(Error::lex(LexError::Syntax(
                    "Unexpected eval or arguments in strict mode".into(),
                    match cursor.peek(0, interner)? {
                        Some(token) => token.span().end(),
                        None => Position::new(1, 1),
                    },
                )));
            }
        }

        let params_start_position = cursor
            .expect(Punctuator::OpenParen, "generator expression", interner)?
            .span()
            .end();

        let params = FormalParameters::new(true, false).parse(cursor, interner)?;

        cursor.expect(Punctuator::CloseParen, "generator expression", interner)?;
        cursor.expect(Punctuator::OpenBlock, "generator expression", interner)?;

        let body = FunctionBody::new(true, false).parse(cursor, interner)?;

        cursor.expect(Punctuator::CloseBlock, "generator expression", interner)?;

        // If the source text matched by FormalParameters is strict mode code,
        // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
        // https://tc39.es/ecma262/#sec-generator-function-definitions-static-semantics-early-errors
        if (cursor.strict_mode() || body.strict()) && params.has_duplicates() {
            return Err(Error::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if FunctionBodyContainsUseStrict of GeneratorBody is true
        // and IsSimpleParameterList of FormalParameters is false.
        // https://tc39.es/ecma262/#sec-generator-function-definitions-static-semantics-early-errors
        if body.strict() && !params.is_simple() {
            return Err(Error::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of GeneratorBody.
        // https://tc39.es/ecma262/#sec-generator-function-definitions-static-semantics-early-errors
        name_in_lexically_declared_names(
            &bound_names(&params),
            &top_level_lexically_declared_names(&body),
            params_start_position,
        )?;

        // It is a Syntax Error if FormalParameters Contains YieldExpression is true.
        // https://tc39.es/ecma262/#sec-generator-function-definitions-static-semantics-early-errors
        if contains(&params, ContainsSymbol::YieldExpression) {
            return Err(Error::lex(LexError::Syntax(
                "generator expression cannot contain yield expression in parameters".into(),
                params_start_position,
            )));
        }

        let function = Generator::new(name, params, body, has_binding_identifier);

        if contains(&function, ContainsSymbol::Super) {
            return Err(Error::lex(LexError::Syntax(
                "invalid super usage".into(),
                params_start_position,
            )));
        }

        Ok(function)
    }
}
