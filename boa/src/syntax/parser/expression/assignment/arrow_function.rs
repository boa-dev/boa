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
    syntax::{
        ast::{
            node::{
                declaration::Declaration, ArrowFunctionDecl, FormalParameter, Node, Return,
                StatementList,
            },
            Punctuator,
        },
        lexer::{Error as LexError, Position, TokenKind},
        parser::{
            error::{ErrorContext, ParseError, ParseResult},
            function::{FormalParameterList, FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            AllowAwait, AllowIn, AllowYield, Cursor, TokenParser,
        },
    },
    BoaProfiler, Interner,
};

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
pub(in crate::syntax::parser) struct ArrowFunction {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ArrowFunction {
    /// Creates a new `ArrowFunction` parser.
    pub(in crate::syntax::parser) fn new<I, Y, A>(
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        I: Into<AllowIn>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
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
    type Output = ArrowFunctionDecl;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ArrowFunction", "Parsing");
        let next_token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        let (params, params_start_position) =
            if let TokenKind::Punctuator(Punctuator::OpenParen) = &next_token.kind() {
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
                    FormalParameterList {
                        parameters: Box::new([FormalParameter::new(
                            Declaration::new_with_identifier(param, None),
                            false,
                        )]),
                        is_simple: true,
                        has_duplicates: false,
                    },
                    params_start_position,
                )
            };

        cursor.peek_expect_no_lineterminator(0, "arrow function", interner)?;

        cursor.expect(
            TokenKind::Punctuator(Punctuator::Arrow),
            "arrow function",
            interner,
        )?;
        let body = ConciseBody::new(self.allow_in).parse(cursor, interner)?;

        // Early Error: ArrowFormalParameters are UniqueFormalParameters.
        if params.has_duplicates {
            return Err(ParseError::lex(LexError::Syntax(
                "Duplicate parameter name not allowed in this context".into(),
                params_start_position,
            )));
        }

        // Early Error: It is a Syntax Error if ConciseBodyContainsUseStrict of ConciseBody is true
        // and IsSimpleParameterList of ArrowParameters is false.
        if body.strict() && !params.is_simple {
            return Err(ParseError::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if any element of the BoundNames of ArrowParameters
        // also occurs in the LexicallyDeclaredNames of ConciseBody.
        // https://tc39.es/ecma262/#sec-arrow-function-definitions-static-semantics-early-errors
        {
            let lexically_declared_names = body.lexically_declared_names(interner);
            for param in params.parameters.as_ref() {
                for param_name in param.names().into_iter() {
                    if lexically_declared_names.contains(&param_name) {
                        return Err(ParseError::lex(LexError::Syntax(
                            format!(
                                "Redeclaration of formal parameter `{}`",
                                interner.resolve(param_name).expect("string disappeared")
                            )
                            .into(),
                            match cursor.peek(0, interner)? {
                                Some(token) => token.span().end(),
                                None => Position::new(1, 1),
                            },
                        )));
                    }
                }
            }
        }

        Ok(ArrowFunctionDecl::new(params.parameters, body))
    }
}

/// <https://tc39.es/ecma262/#prod-ConciseBody>
#[derive(Debug, Clone, Copy)]
struct ConciseBody {
    allow_in: AllowIn,
}

impl ConciseBody {
    /// Creates a new `ConcideBody` parser.
    fn new<I>(allow_in: I) -> Self
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let _ = cursor.next(interner)?;
                let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                cursor.expect(Punctuator::CloseBlock, "arrow function", interner)?;
                Ok(body)
            }
            _ => Ok(StatementList::from(vec![Return::new(
                ExpressionBody::new(self.allow_in, false).parse(cursor, interner)?,
                None,
            )
            .into()])),
        }
    }
}

/// <https://tc39.es/ecma262/#prod-ExpressionBody>
#[derive(Debug, Clone, Copy)]
struct ExpressionBody {
    allow_in: AllowIn,
    allow_await: AllowAwait,
}

impl ExpressionBody {
    /// Creates a new `ExpressionBody` parser.
    fn new<I, A>(allow_in: I, allow_await: A) -> Self
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
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        AssignmentExpression::new(self.allow_in, false, self.allow_await).parse(cursor, interner)
    }
}
