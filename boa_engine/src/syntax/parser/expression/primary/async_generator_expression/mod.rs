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

use crate::syntax::{
    ast::{
        node::{function_contains_super, AsyncGeneratorExpr},
        Keyword, Position, Punctuator,
    },
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::BindingIdentifier,
        function::{FormalParameters, FunctionBody},
        Cursor, ParseError, TokenParser,
    },
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
    name: Option<Sym>,
}

impl AsyncGeneratorExpression {
    /// Creates a new `AsyncGeneratorExpression` parser.
    pub(in crate::syntax::parser) fn new<N>(name: N) -> Self
    where
        N: Into<Option<Sym>>,
    {
        Self { name: name.into() }
    }
}

impl<R> TokenParser<R> for AsyncGeneratorExpression
where
    R: Read,
{
    //The below needs to be implemented in ast::node
    type Output = AsyncGeneratorExpr;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
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

        let name = match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            TokenKind::Punctuator(Punctuator::OpenParen) => self.name,
            _ => Some(BindingIdentifier::new(true, true).parse(cursor, interner)?),
        };

        // Early Error: If BindingIdentifier is present and the source code matching BindingIdentifier is strict
        // mode code, it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
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
            .expect(
                Punctuator::OpenParen,
                "async generator expression",
                interner,
            )?
            .span()
            .end();

        let params = FormalParameters::new(true, true).parse(cursor, interner)?;

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
        params.name_in_lexically_declared_names(
            &body.lexically_declared_names_top_level(),
            params_start_position,
        )?;

        if function_contains_super(&body, &params) {
            return Err(ParseError::lex(LexError::Syntax(
                "invalid super usage".into(),
                params_start_position,
            )));
        }

        //implement the below AsyncGeneratorExpr in ast::node
        Ok(AsyncGeneratorExpr::new(name, params, body))
    }
}
