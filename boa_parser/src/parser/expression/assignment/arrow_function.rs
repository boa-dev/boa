//! Arrow function parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
//! [spec]: https://tc39.es/ecma262/#sec-arrow-function-definitions

use super::AssignmentExpression;
use crate::{
    error::{Error, ErrorContext, ParseResult},
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::BindingIdentifier,
        function::{FormalParameters, FunctionBody},
        name_in_lexically_declared_names, AllowAwait, AllowIn, AllowYield, Cursor, OrAbrupt,
        TokenParser,
    },
};
use ast::operations::{bound_names, top_level_lexically_declared_names};
use boa_ast::{
    self as ast,
    declaration::Variable,
    expression::Identifier,
    function::{FormalParameter, FormalParameterList},
    operations::{contains, ContainsSymbol},
    statement::Return,
    Expression, Punctuator, StatementList,
};
use boa_interner::Interner;
use boa_profiler::Profiler;
use std::io::Read;

/// Arrow function parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
/// [spec]: https://tc39.es/ecma262/#prod-ArrowFunction
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct ArrowFunction {
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ArrowFunction {
    /// Creates a new `ArrowFunction` parser.
    pub(in crate::parser) fn new<N, I, Y, A>(
        name: N,
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        N: Into<Option<Identifier>>,
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_in: allow_in.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ArrowFunction
where
    R: Read,
{
    type Output = ast::function::ArrowFunction;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ArrowFunction", "Parsing");
        let next_token = cursor.peek(0, interner).or_abrupt()?;

        let (params, params_start_position) = if let TokenKind::Punctuator(Punctuator::OpenParen) =
            &next_token.kind()
        {
            // CoverParenthesizedExpressionAndArrowParameterList
            let params_start_position = cursor
                .expect(Punctuator::OpenParen, "arrow function", interner)?
                .span()
                .end();

            let params = FormalParameters::new(self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            cursor.expect(Punctuator::CloseParen, "arrow function", interner)?;
            (params, params_start_position)
        } else {
            let params_start_position = next_token.span().start();
            let param = BindingIdentifier::new(self.allow_yield, self.allow_await)
                .parse(cursor, interner)
                .context("arrow function")?;
            (
                FormalParameterList::try_from(FormalParameter::new(
                    Variable::from_identifier(param, None),
                    false,
                ))
                .expect("a single binding identifier without init is always a valid param list"),
                params_start_position,
            )
        };

        cursor.peek_expect_no_lineterminator(0, "arrow function", interner)?;

        cursor.expect(
            TokenKind::Punctuator(Punctuator::Arrow),
            "arrow function",
            interner,
        )?;
        let arrow = cursor.arrow();
        cursor.set_arrow(true);
        let body = ConciseBody::new(self.allow_in).parse(cursor, interner)?;
        cursor.set_arrow(arrow);

        // Early Error: ArrowFormalParameters are UniqueFormalParameters.
        if params.has_duplicates() {
            return Err(Error::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if ArrowParameters Contains YieldExpression is true.
        if contains(&params, ContainsSymbol::YieldExpression) {
            return Err(Error::lex(LexError::Syntax(
                "Yield expression not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if ArrowParameters Contains AwaitExpression is true.
        if contains(&params, ContainsSymbol::AwaitExpression) {
            return Err(Error::lex(LexError::Syntax(
                "Await expression not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if ConciseBodyContainsUseStrict of ConciseBody is true
        // and IsSimpleParameterList of ArrowParameters is false.
        if body.strict() && !params.is_simple() {
            return Err(Error::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        };

        // It is a Syntax Error if any element of the BoundNames of ArrowParameters
        // also occurs in the LexicallyDeclaredNames of ConciseBody.
        // https://tc39.es/ecma262/#sec-arrow-function-definitions-static-semantics-early-errors
        name_in_lexically_declared_names(
            &bound_names(&params),
            &top_level_lexically_declared_names(&body),
            params_start_position,
        )?;

        Ok(ast::function::ArrowFunction::new(self.name, params, body))
    }
}

/// <https://tc39.es/ecma262/#prod-ConciseBody>
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct ConciseBody {
    allow_in: AllowIn,
}

impl ConciseBody {
    /// Creates a new `ConciseBody` parser.
    pub(in crate::parser) fn new<I>(allow_in: I) -> Self
    where
        I: Into<AllowIn>,
    {
        Self {
            allow_in: allow_in.into(),
        }
    }
}

impl<R> TokenParser<R> for ConciseBody
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        match cursor.peek(0, interner).or_abrupt()?.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                cursor.advance(interner);
                let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                cursor.expect(Punctuator::CloseBlock, "arrow function", interner)?;
                Ok(body)
            }
            _ => Ok(StatementList::from(vec![ast::Statement::Return(
                Return::new(
                    ExpressionBody::new(self.allow_in, false)
                        .parse(cursor, interner)?
                        .into(),
                ),
            )
            .into()])),
        }
    }
}

/// <https://tc39.es/ecma262/#prod-ExpressionBody>
#[derive(Debug, Clone, Copy)]
pub(super) struct ExpressionBody {
    allow_in: AllowIn,
    allow_await: AllowAwait,
}

impl ExpressionBody {
    /// Creates a new `ExpressionBody` parser.
    pub(super) fn new<I, A>(allow_in: I, allow_await: A) -> Self
    where
        I: Into<AllowIn>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_in: allow_in.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ExpressionBody
where
    R: Read,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        AssignmentExpression::new(None, self.allow_in, false, self.allow_await)
            .parse(cursor, interner)
    }
}
