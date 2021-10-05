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
use crate::{
    syntax::{
        ast::{
            node::{self, FunctionExpr, Identifier, MethodDefinitionKind, Node, Object},
            Keyword, Punctuator,
        },
        lexer::{Error as LexError, Position, TokenKind},
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

        // IdentifierReference[?Yield, ?Await]
        if let Some(next_token) = cursor.peek(1)? {
            match next_token.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock)
                | TokenKind::Punctuator(Punctuator::Comma) => {
                    let token = cursor.next()?.ok_or(ParseError::AbruptEnd)?;
                    let ident = match token.kind() {
                        TokenKind::Identifier(ident) => Identifier::from(ident.as_ref()),
                        TokenKind::Keyword(Keyword::Yield) if self.allow_yield.0 => {
                            // Early Error: It is a Syntax Error if this production has a [Yield] parameter and StringValue of Identifier is "yield".
                            return Err(ParseError::general(
                                "Unexpected identifier",
                                token.span().start(),
                            ));
                        }
                        TokenKind::Keyword(Keyword::Yield) if !self.allow_yield.0 => {
                            if cursor.strict_mode() {
                                // Early Error: It is a Syntax Error if the code matched by this production is contained in strict mode code.
                                return Err(ParseError::general(
                                    "Unexpected strict mode reserved word",
                                    token.span().start(),
                                ));
                            }
                            Identifier::from("yield")
                        }
                        TokenKind::Keyword(Keyword::Await) if self.allow_await.0 => {
                            // Early Error: It is a Syntax Error if this production has an [Await] parameter and StringValue of Identifier is "await".
                            return Err(ParseError::general(
                                "Unexpected identifier",
                                token.span().start(),
                            ));
                        }
                        TokenKind::Keyword(Keyword::Await) if !self.allow_await.0 => {
                            if cursor.strict_mode() {
                                // Early Error: It is a Syntax Error if the code matched by this production is contained in strict mode code.
                                return Err(ParseError::general(
                                    "Unexpected strict mode reserved word",
                                    token.span().start(),
                                ));
                            }
                            Identifier::from("yield")
                        }
                        _ => {
                            return Err(ParseError::unexpected(
                                token.clone(),
                                "expected IdentifierReference",
                            ));
                        }
                    };
                    return Ok(node::PropertyDefinition::property(
                        ident.clone().as_ref(),
                        ident,
                    ));
                }
                _ => {}
            }
        }

        //  ... AssignmentExpression[+In, ?Yield, ?Await]
        if cursor.next_if(Punctuator::Spread)?.is_some() {
            let node = AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                .parse(cursor)?;
            return Ok(node::PropertyDefinition::SpreadObject(node));
        }

        // MethodDefinition[?Yield, ?Await] -> GeneratorMethod[?Yield, ?Await]
        if cursor.next_if(Punctuator::Mul)?.is_some() {
            let property_name =
                PropertyName::new(self.allow_yield, self.allow_await).parse(cursor)?;

            let params_start_position = cursor
                .expect(Punctuator::OpenParen, "generator method definition")?
                .span()
                .start();
            let params = FormalParameters::new(false, false).parse(cursor)?;
            cursor.expect(Punctuator::CloseParen, "generator method definition")?;

            // Early Error: UniqueFormalParameters : FormalParameters
            if params.has_duplicates {
                return Err(ParseError::lex(LexError::Syntax(
                    "Duplicate parameter name not allowed in this context".into(),
                    params_start_position,
                )));
            }

            cursor.expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                "generator method definition",
            )?;
            let body = FunctionBody::new(true, false).parse(cursor)?;
            cursor.expect(
                TokenKind::Punctuator(Punctuator::CloseBlock),
                "generator method definition",
            )?;

            // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
            // and IsSimpleParameterList of UniqueFormalParameters is false.
            if body.strict() && !params.is_simple {
                return Err(ParseError::lex(LexError::Syntax(
                    "Illegal 'use strict' directive in function with non-simple parameter list"
                        .into(),
                    params_start_position,
                )));
            }

            // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
            // occurs in the LexicallyDeclaredNames of GeneratorBody.
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

            return Ok(node::PropertyDefinition::method_definition(
                MethodDefinitionKind::Generator,
                property_name,
                FunctionExpr::new(None, params.parameters, body),
            ));
        }

        let mut property_name =
            PropertyName::new(self.allow_yield, self.allow_await).parse(cursor)?;

        //  PropertyName[?Yield, ?Await] : AssignmentExpression[+In, ?Yield, ?Await]
        if cursor.next_if(Punctuator::Colon)?.is_some() {
            let value = AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                .parse(cursor)?;
            return Ok(node::PropertyDefinition::property(property_name, value));
        }

        let ordinary_method = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Punctuator(Punctuator::OpenParen);

        match property_name {
            // MethodDefinition[?Yield, ?Await] -> get ClassElementName[?Yield, ?Await] ( ) { FunctionBody[~Yield, ~Await] }
            node::PropertyName::Literal(str) if str.as_ref() == "get" && !ordinary_method => {
                property_name =
                    PropertyName::new(self.allow_yield, self.allow_await).parse(cursor)?;

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenParen),
                    "get method definition",
                )?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    "get method definition",
                )?;

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "get method definition",
                )?;
                let body = FunctionBody::new(false, false).parse(cursor)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "get method definition",
                )?;

                Ok(node::PropertyDefinition::method_definition(
                    MethodDefinitionKind::Get,
                    property_name,
                    FunctionExpr::new(None, [], body),
                ))
            }
            // MethodDefinition[?Yield, ?Await] -> set ClassElementName[?Yield, ?Await] ( PropertySetParameterList ) { FunctionBody[~Yield, ~Await] }
            node::PropertyName::Literal(str) if str.as_ref() == "set" && !ordinary_method => {
                property_name =
                    PropertyName::new(self.allow_yield, self.allow_await).parse(cursor)?;

                let params_start_position = cursor
                    .expect(
                        TokenKind::Punctuator(Punctuator::OpenParen),
                        "set method definition",
                    )?
                    .span()
                    .end();
                let params = FormalParameters::new(false, false).parse(cursor)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    "set method definition",
                )?;
                if params.parameters.len() != 1 {
                    return Err(ParseError::general(
                        "set method definition must have one parameter",
                        params_start_position,
                    ));
                }

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "set method definition",
                )?;
                let body = FunctionBody::new(false, false).parse(cursor)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "set method definition",
                )?;

                // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
                // and IsSimpleParameterList of PropertySetParameterList is false.
                if body.strict() && !params.is_simple {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                Ok(node::PropertyDefinition::method_definition(
                    MethodDefinitionKind::Set,
                    property_name,
                    FunctionExpr::new(None, params.parameters, body),
                ))
            }
            // MethodDefinition[?Yield, ?Await] -> ClassElementName[?Yield, ?Await] ( UniqueFormalParameters[~Yield, ~Await] ) { FunctionBody[~Yield, ~Await] }
            _ => {
                let params_start_position = cursor
                    .expect(
                        TokenKind::Punctuator(Punctuator::OpenParen),
                        "method definition",
                    )?
                    .span()
                    .end();
                let params = FormalParameters::new(false, false).parse(cursor)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    "method definition",
                )?;

                // Early Error: UniqueFormalParameters : FormalParameters
                if params.has_duplicates {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Duplicate parameter name not allowed in this context".into(),
                        params_start_position,
                    )));
                }

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "method definition",
                )?;
                let body = FunctionBody::new(false, false).parse(cursor)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "method definition",
                )?;

                // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
                // and IsSimpleParameterList of UniqueFormalParameters is false.
                if body.strict() && !params.is_simple {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                Ok(node::PropertyDefinition::method_definition(
                    MethodDefinitionKind::Ordinary,
                    property_name,
                    FunctionExpr::new(None, params.parameters, body),
                ))
            }
        }
    }
}

/// Parses a property name.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyName
#[derive(Debug, Clone)]
struct PropertyName {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PropertyName {
    /// Creates a new `PropertyName` parser.
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

impl<R> TokenParser<R> for PropertyName
where
    R: Read,
{
    type Output = node::PropertyName;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("PropertyName", "Parsing");

        // ComputedPropertyName[?Yield, ?Await] -> [ AssignmentExpression[+In, ?Yield, ?Await] ]
        if cursor.next_if(Punctuator::OpenBracket)?.is_some() {
            let node = AssignmentExpression::new(false, self.allow_yield, self.allow_await)
                .parse(cursor)?;
            cursor.expect(Punctuator::CloseBracket, "expected token ']'")?;
            return Ok(node.into());
        }

        // LiteralPropertyName
        Ok(cursor
            .next()?
            .ok_or(ParseError::AbruptEnd)?
            .to_string()
            .into())
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
