//! Object initializer parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer
//! [spec]: https://tc39.es/ecma262/#sec-object-initializer

#[cfg(test)]
mod tests;
use crate::syntax::ast::node::{Identifier, PropertyName};
use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{
            node::{self, FunctionExpr, MethodDefinitionKind, Node, Object},
            Punctuator,
        },
        parser::{
            expression::AssignmentExpression,
            function::{FormalParameters, FunctionBody},
            AllowAwait, AllowIn, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
        },
    },
    BoaProfiler,
};
use std::io::Read;

/// Parses an object literal.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer
/// [spec]: https://tc39.es/ecma262/#prod-ObjectLiteral
#[derive(Debug, Clone, Copy)]
pub(super) struct ObjectLiteral {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ObjectLiteral {
    /// Creates a new `ObjectLiteral` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ObjectLiteral
where
    R: Read,
{
    type Output = Object;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ObjectLiteral", "Parsing");
        let mut elements = Vec::new();

        loop {
            if cursor.next_if(Punctuator::CloseBlock)?.is_some() {
                break;
            }

            elements
                .push(PropertyDefinition::new(self.allow_yield, self.allow_await).parse(cursor)?);

            if cursor.next_if(Punctuator::CloseBlock)?.is_some() {
                break;
            }

            if cursor.next_if(Punctuator::Comma)?.is_none() {
                let next_token = cursor.next()?.ok_or(ParseError::AbruptEnd)?;
                return Err(ParseError::expected(
                    vec![
                        TokenKind::Punctuator(Punctuator::Comma),
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                    ],
                    next_token,
                    "object literal",
                ));
            }
        }

        Ok(Object::from(elements))
    }
}

/// Parses a property definition.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
#[derive(Debug, Clone, Copy)]
struct PropertyDefinition {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PropertyDefinition {
    /// Creates a new `PropertyDefinition` parser.
    fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for PropertyDefinition
where
    R: Read,
{
    type Output = node::PropertyDefinition;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("PropertyDefinition", "Parsing");

        if cursor.next_if(Punctuator::Spread)?.is_some() {
            let node = AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                .parse(cursor)?;
            return Ok(node::PropertyDefinition::SpreadObject(node));
        }

        // ComputedPropertyName
        // https://tc39.es/ecma262/#prod-ComputedPropertyName
        if cursor.next_if(Punctuator::OpenBracket)?.is_some() {
            let node = AssignmentExpression::new(false, self.allow_yield, self.allow_await)
                .parse(cursor)?;
            cursor.expect(Punctuator::CloseBracket, "expected token ']'")?;
            let next_token = cursor.next()?.ok_or(ParseError::AbruptEnd)?;
            match next_token.kind() {
                TokenKind::Punctuator(Punctuator::Colon) => {
                    let val = AssignmentExpression::new(false, self.allow_yield, self.allow_await)
                        .parse(cursor)?;
                    return Ok(node::PropertyDefinition::property(node, val));
                }
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    return MethodDefinition::new(self.allow_yield, self.allow_await, node)
                        .parse(cursor);
                }
                _ => {
                    return Err(ParseError::unexpected(
                        next_token,
                        "expected AssignmentExpression or MethodDefinition",
                    ))
                }
            }
        }

        // Peek for '}' or ',' to indicate shorthand property name
        if let Some(next_token) = cursor.peek(1)? {
            match next_token.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock)
                | TokenKind::Punctuator(Punctuator::Comma) => {
                    let token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
                    if let TokenKind::Identifier(ident) = token.kind() {
                        // ident is both the name and value in a shorthand property
                        let name = ident.to_string();
                        let value = Identifier::from(ident.to_owned());
                        cursor.next()?.expect("token vanished"); // Consume the token.
                        return Ok(node::PropertyDefinition::property(name, value));
                    } else {
                        // Anything besides an identifier is a syntax error
                        return Err(ParseError::unexpected(token.clone(), "object literal"));
                    }
                }
                _ => {}
            }
        }

        let prop_name = cursor.next()?.ok_or(ParseError::AbruptEnd)?.to_string();
        if cursor.next_if(Punctuator::Colon)?.is_some() {
            let val = AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                .parse(cursor)?;
            return Ok(node::PropertyDefinition::property(prop_name, val));
        }

        // TODO GeneratorMethod
        // https://tc39.es/ecma262/#prod-GeneratorMethod

        if prop_name.as_str() == "async" {
            // TODO - AsyncMethod.
            // https://tc39.es/ecma262/#prod-AsyncMethod

            // TODO - AsyncGeneratorMethod
            // https://tc39.es/ecma262/#prod-AsyncGeneratorMethod
        }

        if cursor
            .next_if(TokenKind::Punctuator(Punctuator::OpenParen))?
            .is_some()
            || ["get", "set"].contains(&prop_name.as_str())
        {
            return MethodDefinition::new(self.allow_yield, self.allow_await, prop_name)
                .parse(cursor);
        }

        let pos = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
        Err(ParseError::general("expected property definition", pos))
    }
}

/// Parses a method definition.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-MethodDefinition
#[derive(Debug, Clone)]
struct MethodDefinition {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    identifier: PropertyName,
}

impl MethodDefinition {
    /// Creates a new `MethodDefinition` parser.
    fn new<Y, A, I>(allow_yield: Y, allow_await: A, identifier: I) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        I: Into<PropertyName>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            identifier: identifier.into(),
        }
    }
}

impl<R> TokenParser<R> for MethodDefinition
where
    R: Read,
{
    type Output = node::PropertyDefinition;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("MethodDefinition", "Parsing");

        let (method_kind, prop_name, params) = match self.identifier {
            PropertyName::Literal(ident)
                if ["get", "set"].contains(&ident.as_ref())
                    && matches!(
                        cursor.peek(0)?.map(|t| t.kind()),
                        Some(&TokenKind::Identifier(_))
                            | Some(&TokenKind::Keyword(_))
                            | Some(&TokenKind::BooleanLiteral(_))
                            | Some(&TokenKind::NullLiteral)
                            | Some(&TokenKind::NumericLiteral(_))
                    ) =>
            {
                let prop_name = cursor.next()?.ok_or(ParseError::AbruptEnd)?.to_string();
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenParen),
                    "property method definition",
                )?;
                let first_param = cursor.peek(0)?.expect("current token disappeared").clone();
                let params = FormalParameters::new(false, false).parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "method definition")?;
                if ident.as_ref() == "get" {
                    if !params.is_empty() {
                        return Err(ParseError::unexpected(
                            first_param,
                            "getter functions must have no arguments",
                        ));
                    }
                    (MethodDefinitionKind::Get, prop_name.into(), params)
                } else {
                    if params.len() != 1 {
                        return Err(ParseError::unexpected(
                            first_param,
                            "setter functions must have one argument",
                        ));
                    }
                    (MethodDefinitionKind::Set, prop_name.into(), params)
                }
            }
            PropertyName::Literal(ident)
                if ["get", "set"].contains(&ident.as_ref())
                    && matches!(
                        cursor.peek(0)?.map(|t| t.kind()),
                        Some(&TokenKind::Punctuator(Punctuator::OpenBracket))
                    ) =>
            {
                cursor.expect(Punctuator::OpenBracket, "token vanished")?;
                let prop_name =
                    AssignmentExpression::new(false, self.allow_yield, self.allow_await)
                        .parse(cursor)?;
                cursor.expect(Punctuator::CloseBracket, "expected token ']'")?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenParen),
                    "property method definition",
                )?;
                let first_param = cursor.peek(0)?.expect("current token disappeared").clone();
                let params = FormalParameters::new(false, false).parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "method definition")?;
                if ident.as_ref() == "get" {
                    if !params.is_empty() {
                        return Err(ParseError::unexpected(
                            first_param,
                            "getter functions must have no arguments",
                        ));
                    }
                    (MethodDefinitionKind::Get, prop_name.into(), params)
                } else {
                    if params.len() != 1 {
                        return Err(ParseError::unexpected(
                            first_param,
                            "setter functions must have one argument",
                        ));
                    }
                    (MethodDefinitionKind::Set, prop_name.into(), params)
                }
            }
            prop_name => {
                let params = FormalParameters::new(false, false).parse(cursor)?;
                cursor.expect(Punctuator::CloseParen, "method definition")?;
                (MethodDefinitionKind::Ordinary, prop_name, params)
            }
        };

        cursor.expect(
            TokenKind::Punctuator(Punctuator::OpenBlock),
            "property method definition",
        )?;
        let body = FunctionBody::new(false, false).parse(cursor)?;
        cursor.expect(
            TokenKind::Punctuator(Punctuator::CloseBlock),
            "property method definition",
        )?;

        Ok(node::PropertyDefinition::method_definition(
            method_kind,
            prop_name,
            FunctionExpr::new(None, params, body),
        ))
    }
}

/// Initializer parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Initializer
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct Initializer {
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Initializer {
    /// Creates a new `Initializer` parser.
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

impl<R> TokenParser<R> for Initializer
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>) -> ParseResult {
        let _timer = BoaProfiler::global().start_event("Initializer", "Parsing");

        cursor.expect(Punctuator::Assign, "initializer")?;
        AssignmentExpression::new(self.allow_in, self.allow_yield, self.allow_await).parse(cursor)
    }
}
