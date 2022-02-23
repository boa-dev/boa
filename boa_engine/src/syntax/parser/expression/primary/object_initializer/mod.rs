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

use crate::syntax::{
    ast::{
        node::{
            object::{self, MethodDefinition},
            AsyncFunctionExpr, AsyncGeneratorExpr, FormalParameterList, FunctionExpr,
            GeneratorExpr, Identifier, Node, Object,
        },
        Keyword, Position, Punctuator,
    },
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::AssignmentExpression,
        function::{FormalParameters, FunctionBody},
        AllowAwait, AllowIn, AllowYield, Cursor, ParseError, ParseResult, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use boa_profiler::Profiler;
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

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("ObjectLiteral", "Parsing");
        let mut elements = Vec::new();

        loop {
            if cursor.next_if(Punctuator::CloseBlock, interner)?.is_some() {
                break;
            }

            elements.push(
                PropertyDefinition::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            );

            if cursor.next_if(Punctuator::CloseBlock, interner)?.is_some() {
                break;
            }

            if cursor.next_if(Punctuator::Comma, interner)?.is_none() {
                let next_token = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;
                return Err(ParseError::expected(
                    [",".to_owned(), "}".to_owned()],
                    next_token.to_string(interner),
                    next_token.span(),
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
    type Output = object::PropertyDefinition;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("PropertyDefinition", "Parsing");

        // IdentifierReference[?Yield, ?Await]
        if let Some(next_token) = cursor.peek(1, interner)? {
            if matches!(
                next_token.kind(),
                TokenKind::Punctuator(Punctuator::CloseBlock | Punctuator::Comma)
            ) {
                let token = cursor.next(interner)?.ok_or(ParseError::AbruptEnd)?;
                let ident = match token.kind() {
                    TokenKind::Identifier(ident) => Identifier::new(*ident),
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
                        Identifier::new(Sym::YIELD)
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
                        Identifier::new(Sym::YIELD)
                    }
                    _ => {
                        return Err(ParseError::unexpected(
                            token.to_string(interner),
                            token.span(),
                            "expected IdentifierReference",
                        ));
                    }
                };
                return Ok(object::PropertyDefinition::property(ident.sym(), ident));
            }
        }

        //  ... AssignmentExpression[+In, ?Yield, ?Await]
        if cursor.next_if(Punctuator::Spread, interner)?.is_some() {
            let node = AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            return Ok(object::PropertyDefinition::SpreadObject(node));
        }

        //Async [AsyncMethod, AsyncGeneratorMethod] object methods
        if cursor.next_if(Keyword::Async, interner)?.is_some() {
            cursor.peek_expect_no_lineterminator(0, "Async object methods", interner)?;

            let mul_check = cursor.next_if(Punctuator::Mul, interner)?;
            let property_name =
                PropertyName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

            if mul_check.is_some() {
                // MethodDefinition[?Yield, ?Await] -> AsyncGeneratorMethod[?Yield, ?Await]

                let params_start_position = cursor
                    .expect(
                        Punctuator::OpenParen,
                        "async generator method definition",
                        interner,
                    )?
                    .span()
                    .start();
                let params = FormalParameters::new(true, true).parse(cursor, interner)?;
                cursor.expect(
                    Punctuator::CloseParen,
                    "async generator method definition",
                    interner,
                )?;

                // Early Error: UniqueFormalParameters : FormalParameters
                // NOTE: does not appear to formally be in ECMAScript specs for method
                if params.has_duplicates() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Duplicate parameter name not allowed in this context".into(),
                        params_start_position,
                    )));
                }

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "async generator method definition",
                    interner,
                )?;
                let body = FunctionBody::new(true, true).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "async generator method definition",
                    interner,
                )?;

                // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
                // and IsSimpleParameterList of UniqueFormalParameters is false.
                if body.strict() && !params.is_simple() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
                // occurs in the LexicallyDeclaredNames of GeneratorBody.
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

                return Ok(object::PropertyDefinition::method_definition(
                    MethodDefinition::AsyncGenerator(AsyncGeneratorExpr::new(None, params, body)),
                    property_name,
                ));
            }
            // MethodDefinition[?Yield, ?Await] -> AsyncMethod[?Yield, ?Await]

            let params_start_position = cursor
                .expect(Punctuator::OpenParen, "async method definition", interner)?
                .span()
                .start();
            let params = FormalParameters::new(false, true).parse(cursor, interner)?;
            cursor.expect(Punctuator::CloseParen, "async method definition", interner)?;

            // Early Error: UniqueFormalParameters : FormalParameters
            // NOTE: does not appear to be in ECMAScript specs
            if params.has_duplicates() {
                return Err(ParseError::lex(LexError::Syntax(
                    "Duplicate parameter name not allowed in this context".into(),
                    params_start_position,
                )));
            }

            cursor.expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                "async method definition",
                interner,
            )?;
            let body = FunctionBody::new(true, true).parse(cursor, interner)?;
            cursor.expect(
                TokenKind::Punctuator(Punctuator::CloseBlock),
                "async method definition",
                interner,
            )?;

            // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
            // and IsSimpleParameterList of UniqueFormalParameters is false.
            if body.strict() && !params.is_simple() {
                return Err(ParseError::lex(LexError::Syntax(
                    "Illegal 'use strict' directive in function with non-simple parameter list"
                        .into(),
                    params_start_position,
                )));
            }

            // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
            // occurs in the LexicallyDeclaredNames of GeneratorBody.
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
            return Ok(object::PropertyDefinition::method_definition(
                MethodDefinition::Async(AsyncFunctionExpr::new(None, params, body)),
                property_name,
            ));
        }

        // MethodDefinition[?Yield, ?Await] -> GeneratorMethod[?Yield, ?Await]
        if cursor.next_if(Punctuator::Mul, interner)?.is_some() {
            let property_name =
                PropertyName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

            let params_start_position = cursor
                .expect(
                    Punctuator::OpenParen,
                    "generator method definition",
                    interner,
                )?
                .span()
                .start();
            let params = FormalParameters::new(false, false).parse(cursor, interner)?;
            cursor.expect(
                Punctuator::CloseParen,
                "generator method definition",
                interner,
            )?;

            // Early Error: UniqueFormalParameters : FormalParameters
            // NOTE: does not appear to be in ECMAScript specs for GeneratorMethod
            if params.has_duplicates() {
                return Err(ParseError::lex(LexError::Syntax(
                    "Duplicate parameter name not allowed in this context".into(),
                    params_start_position,
                )));
            }

            cursor.expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                "generator method definition",
                interner,
            )?;
            let body = FunctionBody::new(true, false).parse(cursor, interner)?;
            cursor.expect(
                TokenKind::Punctuator(Punctuator::CloseBlock),
                "generator method definition",
                interner,
            )?;

            // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
            // and IsSimpleParameterList of UniqueFormalParameters is false.
            if body.strict() && !params.is_simple() {
                return Err(ParseError::lex(LexError::Syntax(
                    "Illegal 'use strict' directive in function with non-simple parameter list"
                        .into(),
                    params_start_position,
                )));
            }

            // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
            // occurs in the LexicallyDeclaredNames of GeneratorBody.
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

            return Ok(object::PropertyDefinition::method_definition(
                MethodDefinition::Generator(GeneratorExpr::new(None, params, body)),
                property_name,
            ));
        }

        let mut property_name =
            PropertyName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        //  PropertyName[?Yield, ?Await] : AssignmentExpression[+In, ?Yield, ?Await]
        if cursor.next_if(Punctuator::Colon, interner)?.is_some() {
            let value = AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            return Ok(object::PropertyDefinition::property(property_name, value));
        }

        let ordinary_method = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
            == &TokenKind::Punctuator(Punctuator::OpenParen);

        match property_name {
            // MethodDefinition[?Yield, ?Await] -> get ClassElementName[?Yield, ?Await] ( ) { FunctionBody[~Yield, ~Await] }
            object::PropertyName::Literal(str) if str == Sym::GET && !ordinary_method => {
                property_name = PropertyName::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenParen),
                    "get method definition",
                    interner,
                )?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    "get method definition",
                    interner,
                )?;

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "get method definition",
                    interner,
                )?;
                let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "get method definition",
                    interner,
                )?;

                Ok(object::PropertyDefinition::method_definition(
                    MethodDefinition::Get(FunctionExpr::new(
                        None,
                        FormalParameterList::default(),
                        body,
                    )),
                    property_name,
                ))
            }
            // MethodDefinition[?Yield, ?Await] -> set ClassElementName[?Yield, ?Await] ( PropertySetParameterList ) { FunctionBody[~Yield, ~Await] }
            object::PropertyName::Literal(str) if str == Sym::SET && !ordinary_method => {
                property_name = PropertyName::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;

                let params_start_position = cursor
                    .expect(
                        TokenKind::Punctuator(Punctuator::OpenParen),
                        "set method definition",
                        interner,
                    )?
                    .span()
                    .end();
                let params = FormalParameters::new(false, false).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    "set method definition",
                    interner,
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
                    interner,
                )?;
                let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "set method definition",
                    interner,
                )?;

                // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
                // and IsSimpleParameterList of PropertySetParameterList is false.
                if body.strict() && !params.is_simple() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                Ok(object::PropertyDefinition::method_definition(
                    MethodDefinition::Set(FunctionExpr::new(None, params, body)),
                    property_name,
                ))
            }
            // MethodDefinition[?Yield, ?Await] -> ClassElementName[?Yield, ?Await] ( UniqueFormalParameters[~Yield, ~Await] ) { FunctionBody[~Yield, ~Await] }
            _ => {
                let params_start_position = cursor
                    .expect(
                        TokenKind::Punctuator(Punctuator::OpenParen),
                        "method definition",
                        interner,
                    )?
                    .span()
                    .end();
                let params = FormalParameters::new(false, false).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    "method definition",
                    interner,
                )?;

                // Early Error: UniqueFormalParameters : FormalParameters
                if params.has_duplicates() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Duplicate parameter name not allowed in this context".into(),
                        params_start_position,
                    )));
                }

                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "method definition",
                    interner,
                )?;
                let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "method definition",
                    interner,
                )?;

                // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
                // and IsSimpleParameterList of UniqueFormalParameters is false.
                if body.strict() && !params.is_simple() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                Ok(object::PropertyDefinition::method_definition(
                    MethodDefinition::Ordinary(FunctionExpr::new(None, params, body)),
                    property_name,
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
    type Output = object::PropertyName;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("PropertyName", "Parsing");

        // ComputedPropertyName[?Yield, ?Await] -> [ AssignmentExpression[+In, ?Yield, ?Await] ]
        if cursor.next_if(Punctuator::OpenBracket, interner)?.is_some() {
            let node = AssignmentExpression::new(None, false, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            cursor.expect(Punctuator::CloseBracket, "expected token ']'", interner)?;
            return Ok(node.into());
        }

        // LiteralPropertyName
        Ok(cursor
            .next(interner)?
            .ok_or(ParseError::AbruptEnd)?
            .to_sym(interner)
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
    name: Option<Sym>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Initializer {
    /// Creates a new `Initializer` parser.
    pub(in crate::syntax::parser) fn new<N, I, Y, A>(
        name: N,
        allow_in: I,
        allow_yield: Y,
        allow_await: A,
    ) -> Self
    where
        N: Into<Option<Sym>>,
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

impl<R> TokenParser<R> for Initializer
where
    R: Read,
{
    type Output = Node;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult {
        let _timer = Profiler::global().start_event("Initializer", "Parsing");

        cursor.expect(Punctuator::Assign, "initializer", interner)?;
        AssignmentExpression::new(self.name, self.allow_in, self.allow_yield, self.allow_await)
            .parse(cursor, interner)
    }
}
