//! Async Generator Expression Parser
//!
//! Implements `TokenParser` fo`AsyncGeneratorExpression`on and outputs
//! an `AsyncGeneratorExpr` ast node
//!
//!  More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorExpression
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
    function::AsyncGenerator,
    operations::{bound_names, contains, lexically_declared_names, ContainsSymbol},
    Keyword, Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
use std::io::Read;

/// Async Generator Expression Parsing
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorExpression
#[derive(Debug, Clone, Copy)]
pub(super) struct AsyncGeneratorExpression {
    name: Option<Identifier>,
}

impl AsyncGeneratorExpression {
    /// Creates a new `AsyncGeneratorExpression` parser.
    pub(in crate::parser) fn new<N>(name: N) -> Self
    where
        N: Into<Option<Identifier>>,
    {
        Self { name: name.into() }
    }
}

impl<R> TokenParser<R> for AsyncGeneratorExpression
where
    R: Read,
{
    //The below needs to be implemented in ast::node
    type Output = AsyncGenerator;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("AsyncGeneratorExpression", "Parsing");

        cursor.peek_expect_no_lineterminator(0, "async generator expression", interner)?;
        cursor.expect(
            (Keyword::Function, false),
            "async generator expression",
            interner,
        )?;
        cursor.expect(
            TokenKind::Punctuator(Punctuator::Mul),
            "async generator expression",
            interner,
        )?;

        let token = cursor.peek(0, interner).or_abrupt()?;
        let (name, name_span) = match token.kind() {
            TokenKind::IdentifierName(_)
            | TokenKind::Keyword((
                Keyword::Yield | Keyword::Await | Keyword::Async | Keyword::Of,
                _,
            )) => {
                let span = token.span();
                let name = BindingIdentifier::new(true, true).parse(cursor, interner)?;

                (Some(name), span)
            }
            _ => (None, token.span()),
        };

        let params_start_position = cursor
            .expect(
                Punctuator::OpenParen,
                "async generator expression",
                interner,
            )?
            .span()
            .end();

        let params = FormalParameters::new(true, true).parse(cursor, interner)?;

        // It is a Syntax Error if FormalParameters Contains YieldExpression is true.
        if contains(&params, ContainsSymbol::YieldExpression) {
            return Err(Error::lex(LexError::Syntax(
                "yield expression not allowed in async generator expression parameters".into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if FormalParameters Contains AwaitExpression is true.
        if contains(&params, ContainsSymbol::AwaitExpression) {
            return Err(Error::lex(LexError::Syntax(
                "await expression not allowed in async generator expression parameters".into(),
                params_start_position,
            )));
        }

        cursor.expect(
            Punctuator::CloseParen,
            "async generator expression",
            interner,
        )?;
        cursor.expect(
            Punctuator::OpenBlock,
            "async generator expression",
            interner,
        )?;

        let body = FunctionBody::new(true, true).parse(cursor, interner)?;

        cursor.expect(
            Punctuator::CloseBlock,
            "async generator expression",
            interner,
        )?;

        // Early Error: If the source code matching FormalParameters is strict mode code,
        // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
        if (cursor.strict() || body.strict()) && params.has_duplicates() {
            return Err(Error::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of GeneratorBody is true
        // and IsSimpleParameterList of FormalParameters is false.
        if body.strict() && !params.is_simple() {
            return Err(Error::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        }

        // Early Error: If BindingIdentifier is present and the source code matching BindingIdentifier is strict mode code,
        // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
        if let Some(name) = name {
            if (cursor.strict() || body.strict())
                && [Sym::EVAL, Sym::ARGUMENTS].contains(&name.sym())
            {
                return Err(Error::lex(LexError::Syntax(
                    "unexpected identifier 'eval' or 'arguments' in strict mode".into(),
                    name_span.start(),
                )));
            }
        }

        // Catch early error for BindingIdentifier, because strictness of the functions body is also
        // relevant for the function parameters.
        if body.strict() && contains(&params, ContainsSymbol::EvalOrArguments) {
            return Err(Error::lex(LexError::Syntax(
                "unexpected identifier 'eval' or 'arguments' in strict mode".into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if any element of the BoundNames of FormalParameters
        // also occurs in the LexicallyDeclaredNames of FunctionBody.
        name_in_lexically_declared_names(
            &bound_names(&params),
            &lexically_declared_names(&body),
            params_start_position,
            interner,
        )?;

        let function = AsyncGenerator::new(name.or(self.name), params, body, name.is_some());

        if contains(&function, ContainsSymbol::Super) {
            return Err(Error::lex(LexError::Syntax(
                "invalid super usage".into(),
                params_start_position,
            )));
        }

        Ok(function)
    }
}
