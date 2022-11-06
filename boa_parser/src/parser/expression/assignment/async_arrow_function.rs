//! Async arrow function parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
//! [spec]: https://tc39.es/ecma262/#sec-async-arrow-function-definitions

use super::arrow_function::ExpressionBody;
use crate::{
    error::{Error, ErrorContext, ParseResult},
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::BindingIdentifier,
        function::{FormalParameters, FunctionBody},
        name_in_lexically_declared_names, AllowIn, AllowYield, Cursor, OrAbrupt, TokenParser,
    },
};
use ast::{
    operations::{bound_names, contains, top_level_lexically_declared_names, ContainsSymbol},
    Keyword,
};
use boa_ast::{
    self as ast,
    declaration::Variable,
    expression::Identifier,
    function::{FormalParameter, FormalParameterList},
    statement::Return,
    Punctuator, StatementList,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Async arrow function parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
/// [spec]: https://tc39.es/ecma262/#prod-AsyncArrowFunction
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct AsyncArrowFunction {
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
}

impl AsyncArrowFunction {
    /// Creates a new `AsyncArrowFunction` parser.
    pub(in crate::parser) fn new<N, I, Y>(name: N, allow_in: I, allow_yield: Y) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
    {
        Self {
            name: name.into(),
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
        }
    }
}

impl<R> TokenParser<R> for AsyncArrowFunction
where
    R: Read,
{
    type Output = ast::function::AsyncArrowFunction;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("AsyncArrowFunction", "Parsing");

        cursor.expect((Keyword::Async, false), "async arrow function", interner)?;
        cursor.peek_expect_no_lineterminator(0, "async arrow function", interner)?;

        let next_token = cursor.peek(0, interner).or_abrupt()?;
        let (params, params_start_position) = if let TokenKind::Punctuator(Punctuator::OpenParen) =
            &next_token.kind()
        {
            let params_start_position = cursor
                .expect(Punctuator::OpenParen, "async arrow function", interner)?
                .span()
                .end();

            let params = FormalParameters::new(false, true).parse(cursor, interner)?;
            cursor.expect(Punctuator::CloseParen, "async arrow function", interner)?;
            (params, params_start_position)
        } else {
            let params_start_position = next_token.span().start();
            let param = BindingIdentifier::new(self.allow_yield, true)
                .parse(cursor, interner)
                .context("async arrow function")?;
            (
                FormalParameterList::try_from(FormalParameter::new(
                    Variable::from_identifier(param, None),
                    false,
                ))
                .expect("a single binding identifier without init is always a valid param list"),
                params_start_position,
            )
        };

        cursor.peek_expect_no_lineterminator(0, "async arrow function", interner)?;
        cursor.expect(Punctuator::Arrow, "async arrow function", interner)?;

        let body = AsyncConciseBody::new(self.allow_in).parse(cursor, interner)?;

        // Early Error: ArrowFormalParameters are UniqueFormalParameters.
        if params.has_duplicates() {
            return Err(Error::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if CoverCallExpressionAndAsyncArrowHead Contains YieldExpression is true.
        if contains(&params, ContainsSymbol::YieldExpression) {
            return Err(Error::lex(LexError::Syntax(
                "Yield expression not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if CoverCallExpressionAndAsyncArrowHead Contains AwaitExpression is true.
        if contains(&params, ContainsSymbol::AwaitExpression) {
            return Err(Error::lex(LexError::Syntax(
                "Await expression not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if AsyncConciseBodyContainsUseStrict of AsyncConciseBody is true and
        // IsSimpleParameterList of CoverCallExpressionAndAsyncArrowHead is false.
        if body.strict() && !params.is_simple() {
            return Err(Error::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of CoverCallExpressionAndAsyncArrowHead
        // also occurs in the LexicallyDeclaredNames of AsyncConciseBody.
        name_in_lexically_declared_names(
            &bound_names(&params),
            &top_level_lexically_declared_names(&body),
            params_start_position,
        )?;

        Ok(ast::function::AsyncArrowFunction::new(
            self.name, params, body,
        ))
    }
}

/// <https://tc39.es/ecma262/#prod-AsyncConciseBody>
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct AsyncConciseBody {
    allow_in: AllowIn,
}

impl AsyncConciseBody {
    /// Creates a new `AsyncConciseBody` parser.
    pub(in crate::parser) fn new<I>(allow_in: I) -> Self
    where
        I: Into<AllowIn>,
    {
        Self {
            allow_in: allow_in.into(),
        }
    }
}

impl<R> TokenParser<R> for AsyncConciseBody
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        match cursor.peek(0, interner).or_abrupt()?.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                cursor.advance(interner);
                let body = FunctionBody::new(false, true).parse(cursor, interner)?;
                cursor.expect(Punctuator::CloseBlock, "async arrow function", interner)?;
                Ok(body)
            }
            _ => Ok(StatementList::from(vec![ast::Statement::Return(
                Return::new(
                    ExpressionBody::new(self.allow_in, true)
                        .parse(cursor, interner)?
                        .into(),
                ),
            )
            .into()])),
        }
    }
}
