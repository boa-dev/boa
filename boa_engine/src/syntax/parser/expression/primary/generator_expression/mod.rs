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

use crate::syntax::{
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::BindingIdentifier,
        function::{FormalParameters, FunctionBody},
        function_contains_super, name_in_lexically_declared_names, Cursor, ParseError, ParseResult,
        TokenParser,
    },
};
use boa_ast::{expression::Identifier, function::Generator, Position, Punctuator};
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
    pub(in crate::syntax::parser) fn new<N>(name: N) -> Self
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

        let name = match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            TokenKind::Punctuator(Punctuator::OpenParen) => self.name,
            _ => Some(BindingIdentifier::new(true, false).parse(cursor, interner)?),
        };

        // Early Error: If BindingIdentifier is present and the source code matching BindingIdentifier is strict mode code,
        // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
        if let Some(name) = name {
            if cursor.strict_mode() && [Sym::EVAL, Sym::ARGUMENTS].contains(&name.sym()) {
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
            .expect(Punctuator::OpenParen, "generator expression", interner)?
            .span()
            .end();

        let params = FormalParameters::new(true, false).parse(cursor, interner)?;

        cursor.expect(Punctuator::CloseParen, "generator expression", interner)?;
        cursor.expect(Punctuator::OpenBlock, "generator expression", interner)?;

        let body = FunctionBody::new(true, false).parse(cursor, interner)?;

        cursor.expect(Punctuator::CloseBlock, "generator expression", interner)?;

        // Early Error: If the source code matching FormalParameters is strict mode code,
        // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
        if (cursor.strict_mode() || body.strict()) && params.has_duplicates() {
            return Err(ParseError::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of GeneratorBody is true
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
        name_in_lexically_declared_names(
            &params,
            &body.lexically_declared_names_top_level(),
            params_start_position,
        )?;

        if function_contains_super(&body, &params) {
            return Err(ParseError::lex(LexError::Syntax(
                "invalid super usage".into(),
                params_start_position,
            )));
        }

        Ok(Generator::new(name, params, body))
    }
}
