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
            GeneratorExpr, Node, Object,
        },
        Const, Keyword, Punctuator,
    },
    lexer::{token::Numeric, Error as LexError, TokenKind},
    parser::{
        expression::{identifiers::IdentifierReference, AssignmentExpression},
        function::{FormalParameter, FormalParameters, FunctionBody, UniqueFormalParameters},
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
pub(in crate::syntax::parser) struct PropertyDefinition {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PropertyDefinition {
    /// Creates a new `PropertyDefinition` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
                let ident = IdentifierReference::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
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
        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        match token.kind() {
            TokenKind::Keyword((Keyword::Async, true)) => {
                return Err(ParseError::general(
                    "Keyword must not contain escaped characters",
                    token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Async, false)) => {
                cursor.next(interner)?.expect("token disappeared");
                cursor.peek_expect_no_lineterminator(0, "Async object methods", interner)?;

                let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                if let TokenKind::Punctuator(Punctuator::Mul) = token.kind() {
                    let (property_name, method) =
                        AsyncGeneratorMethod::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                    return Ok(object::PropertyDefinition::method_definition(
                        method,
                        property_name,
                    ));
                }
                let (property_name, method) =
                    AsyncMethod::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;
                return Ok(object::PropertyDefinition::method_definition(
                    method,
                    property_name,
                ));
            }
            _ => {}
        }

        if cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
            == &TokenKind::Punctuator(Punctuator::Mul)
        {
            let position = cursor
                .peek(0, interner)?
                .ok_or(ParseError::AbruptEnd)?
                .span()
                .start();
            let (class_element_name, method) =
                GeneratorMethod::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

            match class_element_name {
                object::ClassElementName::PropertyName(property_name) => {
                    return Ok(object::PropertyDefinition::method_definition(
                        method,
                        property_name,
                    ))
                }
                object::ClassElementName::PrivateIdentifier(_) => {
                    return Err(ParseError::general(
                        "private identifier not allowed in object literal",
                        position,
                    ))
                }
            }
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
                let parameters: FormalParameterList = FormalParameter::new(false, false)
                    .parse(cursor, interner)?
                    .into();
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseParen),
                    "set method definition",
                    interner,
                )?;

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
                if body.strict() && !parameters.is_simple() {
                    return Err(ParseError::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                Ok(object::PropertyDefinition::method_definition(
                    MethodDefinition::Set(FunctionExpr::new(None, parameters, body)),
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

                // It is a Syntax Error if any element of the BoundNames of FormalParameters also occurs in the LexicallyDeclaredNames of FunctionBody.
                let lexically_declared_names = body.lexically_declared_names();
                for parameter in params.parameters.iter() {
                    for name in &parameter.names() {
                        if lexically_declared_names.contains(&(*name, false)) {
                            return Err(ParseError::general(
                                "formal parameter declared in lexically declared names",
                                params_start_position,
                            ));
                        }
                        if lexically_declared_names.contains(&(*name, true)) {
                            return Err(ParseError::general(
                                "formal parameter declared in lexically declared names",
                                params_start_position,
                            ));
                        }
                    }
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
pub(in crate::syntax::parser) struct PropertyName {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PropertyName {
    /// Creates a new `PropertyName` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let name = match token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                cursor.next(interner).expect("token disappeared");
                let node =
                    AssignmentExpression::new(None, false, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                cursor.expect(Punctuator::CloseBracket, "expected token ']'", interner)?;
                return Ok(node.into());
            }
            TokenKind::Identifier(name) => object::PropertyName::Literal(*name),
            TokenKind::StringLiteral(name) => Node::Const(Const::from(*name)).into(),
            TokenKind::NumericLiteral(num) => match num {
                Numeric::Rational(num) => Node::Const(Const::from(*num)).into(),
                Numeric::Integer(num) => Node::Const(Const::from(*num)).into(),
                Numeric::BigInt(num) => Node::Const(Const::from(num.clone())).into(),
            },
            TokenKind::Keyword((word, _)) => {
                Node::Const(Const::from(interner.get_or_intern_static(word.as_str()))).into()
            }
            TokenKind::NullLiteral => Node::Const(Const::from(Sym::NULL)).into(),
            TokenKind::BooleanLiteral(bool) => match bool {
                true => Node::Const(Const::from(interner.get_or_intern_static("true"))).into(),
                false => Node::Const(Const::from(interner.get_or_intern_static("false"))).into(),
            },
            _ => return Err(ParseError::AbruptEnd),
        };
        cursor.next(interner).expect("token disappeared");
        Ok(name)
    }
}

/// `ClassElementName` can be either a property name or a private identifier.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElementName
#[derive(Debug, Clone)]
pub(in crate::syntax::parser) struct ClassElementName {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassElementName {
    /// Creates a new `ClassElementName` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for ClassElementName
where
    R: Read,
{
    type Output = object::ClassElementName;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("ClassElementName", "Parsing");

        match cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
        {
            TokenKind::PrivateIdentifier(ident) => {
                let ident = *ident;
                cursor.next(interner).expect("token disappeared");
                Ok(object::ClassElementName::PrivateIdentifier(ident))
            }
            _ => Ok(object::ClassElementName::PropertyName(
                PropertyName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?,
            )),
        }
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

/// `GeneratorMethod` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-GeneratorMethod
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct GeneratorMethod {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl GeneratorMethod {
    /// Creates a new `GeneratorMethod` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for GeneratorMethod
where
    R: Read,
{
    type Output = (object::ClassElementName, MethodDefinition);

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("GeneratorMethod", "Parsing");
        cursor.expect(Punctuator::Mul, "generator method definition", interner)?;

        let class_element_name =
            ClassElementName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let params = UniqueFormalParameters::new(true, false).parse(cursor, interner)?;

        let body_start = cursor
            .expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                "generator method definition",
                interner,
            )?
            .span()
            .start();
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
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                body_start,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
        // occurs in the LexicallyDeclaredNames of GeneratorBody.
        params.name_in_lexically_declared_names(
            &body.lexically_declared_names_top_level(),
            body_start,
        )?;

        Ok((
            class_element_name,
            MethodDefinition::Generator(GeneratorExpr::new(None, params, body)),
        ))
    }
}

/// `AsyncGeneratorMethod` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorMethod
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct AsyncGeneratorMethod {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AsyncGeneratorMethod {
    /// Creates a new `AsyncGeneratorMethod` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for AsyncGeneratorMethod
where
    R: Read,
{
    type Output = (object::PropertyName, MethodDefinition);

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("AsyncGeneratorMethod", "Parsing");
        cursor.expect(
            Punctuator::Mul,
            "async generator method definition",
            interner,
        )?;

        let property_name =
            PropertyName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let params = UniqueFormalParameters::new(true, true).parse(cursor, interner)?;

        let body_start = cursor
            .expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                "async generator method definition",
                interner,
            )?
            .span()
            .start();
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
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                body_start,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
        // occurs in the LexicallyDeclaredNames of GeneratorBody.
        params.name_in_lexically_declared_names(
            &body.lexically_declared_names_top_level(),
            body_start,
        )?;

        Ok((
            property_name,
            MethodDefinition::AsyncGenerator(AsyncGeneratorExpr::new(None, params, body)),
        ))
    }
}

/// `AsyncMethod` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncMethod
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct AsyncMethod {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AsyncMethod {
    /// Creates a new `AsyncMethod` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for AsyncMethod
where
    R: Read,
{
    type Output = (object::PropertyName, MethodDefinition);

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let _timer = Profiler::global().start_event("AsyncMethod", "Parsing");

        let property_name =
            PropertyName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let params = UniqueFormalParameters::new(false, true).parse(cursor, interner)?;

        let body_start = cursor
            .expect(
                TokenKind::Punctuator(Punctuator::OpenBlock),
                "async method definition",
                interner,
            )?
            .span()
            .start();
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
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                body_start,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
        // occurs in the LexicallyDeclaredNames of GeneratorBody.
        params.name_in_lexically_declared_names(
            &body.lexically_declared_names_top_level(),
            body_start,
        )?;

        Ok((
            property_name,
            MethodDefinition::Async(AsyncFunctionExpr::new(None, params, body)),
        ))
    }
}
