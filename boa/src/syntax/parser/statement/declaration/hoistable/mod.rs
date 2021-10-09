//! Hoistable declaration parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-HoistableDeclaration

#[cfg(test)]
mod tests;

mod async_function_decl;
mod function_decl;
mod generator_decl;

use async_function_decl::AsyncFunctionDeclaration;
pub(in crate::syntax::parser) use function_decl::FunctionDeclaration;
use generator_decl::GeneratorDeclaration;

use crate::{
    syntax::{
        ast::node::{FormalParameter, StatementList},
        ast::{Keyword, Node, Punctuator},
        lexer::{Position, TokenKind},
        parser::{
            function::{FormalParameters, FunctionStatementList},
            statement::{BindingIdentifier, LexError},
            Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};
use std::io::Read;

/// Hoistable declaration parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct HoistableDeclaration<const YIELD: bool, const AWAIT: bool, const DEFAULT: bool>;

impl<R, const YIELD: bool, const AWAIT: bool, const DEFAULT: bool> TokenParser<R>
    for HoistableDeclaration<YIELD, AWAIT, DEFAULT>
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("HoistableDeclaration", "Parsing");
        let tok = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function) => {
                let next_token = cursor.peek(1)?.ok_or(ParseError::AbruptEnd)?;
                if let TokenKind::Punctuator(Punctuator::Mul) = next_token.kind() {
                    GeneratorDeclaration::<YIELD, AWAIT, DEFAULT>
                        .parse(cursor)
                        .map(Node::from)
                } else {
                    FunctionDeclaration::<YIELD, AWAIT, DEFAULT>
                        .parse(cursor)
                        .map(Node::from)
                }
            }
            TokenKind::Keyword(Keyword::Async) => AsyncFunctionDeclaration::<YIELD, AWAIT, false>
                .parse(cursor)
                .map(Node::from),
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}

// This is a helper function to not duplicate code in the individual callable deceleration parsers.
#[inline]
#[allow(clippy::type_complexity)]
fn parse_callable_declaration<
    R: Read,
    const IDENT_YIELD: bool,
    const IDENT_AWAIT: bool,
    const DEFAULT: bool,
    const BODY_YIELD: bool,
    const BODY_AWAIT: bool,
>(
    error_context: &'static str,
    cursor: &mut Cursor<R>,
) -> Result<(Box<str>, Box<[FormalParameter]>, StatementList), ParseError> {
    let next_token = cursor.peek(0)?;
    let name = if let Some(token) = next_token {
        match token.kind() {
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                if !DEFAULT {
                    return Err(ParseError::unexpected(token.clone(), error_context));
                }
                "default".into()
            }
            _ => BindingIdentifier::<IDENT_YIELD, IDENT_AWAIT>.parse(cursor)?,
        }
    } else {
        return Err(ParseError::AbruptEnd);
    };

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
        .expect(Punctuator::OpenParen, error_context)?
        .span()
        .end();

    let params = FormalParameters::<BODY_YIELD, BODY_AWAIT>.parse(cursor)?;

    cursor.expect(Punctuator::CloseParen, error_context)?;
    cursor.expect(Punctuator::OpenBlock, error_context)?;

    let body = FunctionStatementList::<BODY_YIELD, BODY_AWAIT>.parse(cursor)?;

    cursor.expect(Punctuator::CloseBlock, error_context)?;

    // Early Error: If the source code matching FormalParameters is strict mode code,
    // the Early Error rules for UniqueFormalParameters : FormalParameters are applied.
    if (cursor.strict_mode() || body.strict()) && params.has_duplicates {
        return Err(ParseError::lex(LexError::Syntax(
            "Duplicate parameter name not allowed in this context".into(),
            params_start_position,
        )));
    }

    // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
    // and IsSimpleParameterList of FormalParameters is false.
    if body.strict() && !params.is_simple {
        return Err(ParseError::lex(LexError::Syntax(
            "Illegal 'use strict' directive in function with non-simple parameter list".into(),
            params_start_position,
        )));
    }

    // It is a Syntax Error if any element of the BoundNames of FormalParameters
    // also occurs in the LexicallyDeclaredNames of FunctionBody.
    // https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors
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

    Ok((name, params.parameters, body))
}
