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
            node::{ArrowFunctionDecl, FormalParameter, Node, Return, StatementList},
            Punctuator,
        },
        lexer::{Error as LexError, Position, TokenKind},
        parser::{
            error::{ErrorContext, ParseError, ParseResult},
            function::{FormalParameterList, FormalParameters, FunctionStatementList},
            statement::BindingIdentifier,
            Cursor, TokenParser,
        },
    },
    BoaProfiler,
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
pub(in crate::syntax::parser) struct ArrowFunction<
    const IN: bool,
    const YIELD: bool,
    const AWAIT: bool,
>;

impl<R, const IN: bool, const YIELD: bool, const AWAIT: bool> TokenParser<R>
    for ArrowFunction<IN, YIELD, AWAIT>
where
    R: Read,
{
    type Output = ArrowFunctionDecl;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ArrowFunction", "Parsing");
        let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        let (params, params_start_position) =
            if let TokenKind::Punctuator(Punctuator::OpenParen) = &next_token.kind() {
                // CoverParenthesizedExpressionAndArrowParameterList
                let params_start_position = cursor
                    .expect(Punctuator::OpenParen, "arrow function")?
                    .span()
                    .end();

                let params = FormalParameters::<YIELD, AWAIT>.parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "arrow function")?;
                (params, params_start_position)
            } else {
                let params_start_position = next_token.span().start();
                let param = BindingIdentifier::<YIELD, AWAIT>
                    .parse(cursor)
                    .context("arrow function")?;
                (
                    FormalParameterList {
                        parameters: Box::new([FormalParameter::new(param, None, false)]),
                        is_simple: true,
                        has_duplicates: false,
                    },
                    params_start_position,
                )
            };

        cursor.peek_expect_no_lineterminator(0, "arrow function")?;

        cursor.expect(TokenKind::Punctuator(Punctuator::Arrow), "arrow function")?;
        let body = ConciseBody::<IN>.parse(cursor)?;

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

        Ok(ArrowFunctionDecl::new(params.parameters, body))
    }
}

/// <https://tc39.es/ecma262/#prod-ConciseBody>
#[derive(Debug, Clone, Copy)]
struct ConciseBody<const IN: bool>;

impl<R, const IN: bool> TokenParser<R> for ConciseBody<IN>
where
    R: Read,
{
    type Output = StatementList;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        match cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind() {
            TokenKind::Punctuator(Punctuator::OpenBlock) => {
                let _ = cursor.next();
                let body = FunctionStatementList::<false, false>.parse(cursor)?;
                cursor.expect(Punctuator::CloseBlock, "arrow function")?;
                Ok(body)
            }
            _ => Ok(StatementList::from(vec![Return::new(
                ExpressionBody::<IN, false>.parse(cursor)?,
                None,
            )
            .into()])),
        }
    }
}

/// <https://tc39.es/ecma262/#prod-ExpressionBody>
#[derive(Debug, Clone, Copy)]
struct ExpressionBody<const IN: bool, const AWAIT: bool>;

impl<R, const IN: bool, const AWAIT: bool> TokenParser<R> for ExpressionBody<IN, AWAIT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        AssignmentExpression::<IN, false, AWAIT>.parse(cursor)
    }
}
