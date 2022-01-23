//! Hoistable declaration parsing.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#prod-HoistableDeclaration

#[cfg(test)]
mod tests;

mod async_function_decl;
mod async_generator_decl;
mod function_decl;
mod generator_decl;

use async_function_decl::AsyncFunctionDeclaration;
use async_generator_decl::AsyncGeneratorDeclaration;
pub(in crate::syntax::parser) use function_decl::FunctionDeclaration;
use generator_decl::GeneratorDeclaration;

use crate::{
    syntax::{
        ast::node::{FormalParameter, StatementList},
        ast::{Keyword, Node, Punctuator},
        lexer::{Position, TokenKind},
        parser::{
            function::{FormalParameters, FunctionBody},
            statement::{BindingIdentifier, LexError},
            AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};
use boa_interner::{Interner, Sym};
use std::io::Read;

/// Hoistable declaration parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FunctionDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct HoistableDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl HoistableDeclaration {
    /// Creates a new `HoistableDeclaration` parser.
    pub(super) fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        D: Into<AllowDefault>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            is_default: is_default.into(),
        }
    }
}

impl<R> TokenParser<R> for HoistableDeclaration
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("HoistableDeclaration", "Parsing");
        let tok = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;

        match tok.kind() {
            TokenKind::Keyword(Keyword::Function) => {
                let next_token = cursor.peek(1, interner)?.ok_or(ParseError::AbruptEnd)?;
                if let TokenKind::Punctuator(Punctuator::Mul) = next_token.kind() {
                    GeneratorDeclaration::new(self.allow_yield, self.allow_await, self.is_default)
                        .parse(cursor, interner)
                        .map(Node::from)
                } else {
                    FunctionDeclaration::new(self.allow_yield, self.allow_await, self.is_default)
                        .parse(cursor, interner)
                        .map(Node::from)
                }
            }
            TokenKind::Keyword(Keyword::Async) => {
                let next_token = cursor.peek(2, interner)?.ok_or(ParseError::AbruptEnd)?;
                if let TokenKind::Punctuator(Punctuator::Mul) = next_token.kind() {
                    AsyncGeneratorDeclaration::new(
                        self.allow_yield,
                        self.allow_await,
                        self.is_default,
                    )
                    .parse(cursor, interner)
                    .map(Node::from)
                } else {
                    AsyncFunctionDeclaration::new(self.allow_yield, self.allow_await, false)
                        .parse(cursor, interner)
                        .map(Node::from)
                }
            }
            _ => unreachable!("unknown token found: {:?}", tok),
        }
    }
}

trait CallableDeclaration {
    fn error_context(&self) -> &'static str;
    fn is_default(&self) -> bool;
    fn name_allow_yield(&self) -> bool;
    fn name_allow_await(&self) -> bool;
    fn parameters_allow_yield(&self) -> bool;
    fn parameters_allow_await(&self) -> bool;
    fn body_allow_yield(&self) -> bool;
    fn body_allow_await(&self) -> bool;
}

// This is a helper function to not duplicate code in the individual callable deceleration parsers.
#[inline]
#[allow(clippy::type_complexity)]
fn parse_callable_declaration<R: Read, C: CallableDeclaration>(
    c: &C,
    cursor: &mut Cursor<R>,
    interner: &mut Interner,
) -> Result<(Sym, Box<[FormalParameter]>, StatementList), ParseError> {
    let next_token = cursor.peek(0, interner)?;
    let name = if let Some(token) = next_token {
        match token.kind() {
            TokenKind::Punctuator(Punctuator::OpenParen) => {
                if !c.is_default() {
                    return Err(ParseError::unexpected(
                        token.to_string(interner),
                        token.span(),
                        c.error_context(),
                    ));
                }
                Sym::DEFAULT
            }
            _ => BindingIdentifier::new(c.name_allow_yield(), c.name_allow_await())
                .parse(cursor, interner)?,
        }
    } else {
        return Err(ParseError::AbruptEnd);
    };

    // Early Error: If BindingIdentifier is present and the source code matching BindingIdentifier is strict mode code,
    // it is a Syntax Error if the StringValue of BindingIdentifier is "eval" or "arguments".
    if cursor.strict_mode() && [Sym::EVAL, Sym::ARGUMENTS].contains(&name) {
        return Err(ParseError::lex(LexError::Syntax(
            "Unexpected eval or arguments in strict mode".into(),
            match cursor.peek(0, interner)? {
                Some(token) => token.span().end(),
                None => Position::new(1, 1),
            },
        )));
    }

    let params_start_position = cursor
        .expect(Punctuator::OpenParen, c.error_context(), interner)?
        .span()
        .end();

    let params = FormalParameters::new(c.parameters_allow_yield(), c.parameters_allow_await())
        .parse(cursor, interner)?;

    cursor.expect(Punctuator::CloseParen, c.error_context(), interner)?;
    cursor.expect(Punctuator::OpenBlock, c.error_context(), interner)?;

    let body =
        FunctionBody::new(c.body_allow_yield(), c.body_allow_await()).parse(cursor, interner)?;

    cursor.expect(Punctuator::CloseBlock, c.error_context(), interner)?;

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
        let lexically_declared_names = body.lexically_declared_names(interner);
        for param in params.parameters.as_ref() {
            for param_name in param.names() {
                if lexically_declared_names.contains(&param_name) {
                    return Err(ParseError::lex(LexError::Syntax(
                        format!(
                            "Redeclaration of formal parameter `{}`",
                            interner.resolve_expect(param_name)
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

    Ok((name, params.parameters, body))
}
