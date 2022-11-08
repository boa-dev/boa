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
    lexer::{token::Numeric, Error as LexError, TokenKind},
    parser::{
        expression::{identifiers::IdentifierReference, AssignmentExpression},
        function::{FormalParameter, FormalParameters, FunctionBody, UniqueFormalParameters},
        name_in_lexically_declared_names, AllowAwait, AllowIn, AllowYield, Cursor, OrAbrupt,
        ParseResult, TokenParser,
    },
    Error,
};
use boa_ast::{
    expression::{
        literal::{self, Literal},
        Identifier,
    },
    function::{AsyncFunction, AsyncGenerator, FormalParameterList, Function, Generator},
    operations::{
        bound_names, contains, has_direct_super, top_level_lexically_declared_names, ContainsSymbol,
    },
    property::{self, MethodDefinition},
    Expression, Keyword, Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;
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
    type Output = literal::ObjectLiteral;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
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
                let next_token = cursor.next(interner).or_abrupt()?;
                return Err(Error::expected(
                    [",".to_owned(), "}".to_owned()],
                    next_token.to_string(interner),
                    next_token.span(),
                    "object literal",
                ));
            }
        }

        Ok(literal::ObjectLiteral::from(elements))
    }
}

/// Parses a property definition.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyDefinition
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct PropertyDefinition {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PropertyDefinition {
    /// Creates a new `PropertyDefinition` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = property::PropertyDefinition;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("PropertyDefinition", "Parsing");

        match cursor.peek(1, interner).or_abrupt()?.kind() {
            TokenKind::Punctuator(Punctuator::CloseBlock | Punctuator::Comma) => {
                let ident = IdentifierReference::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                return Ok(property::PropertyDefinition::Property(
                    ident.sym().into(),
                    ident.into(),
                ));
            }
            TokenKind::Punctuator(Punctuator::Assign) => {
                return CoverInitializedName::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner);
            }
            _ => {}
        }

        //  ... AssignmentExpression[+In, ?Yield, ?Await]
        if cursor.next_if(Punctuator::Spread, interner)?.is_some() {
            let node = AssignmentExpression::new(None, true, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            return Ok(property::PropertyDefinition::SpreadObject(node));
        }

        //Async [AsyncMethod, AsyncGeneratorMethod] object methods
        let is_keyword = !matches!(
            cursor.peek(1, interner).or_abrupt()?.kind(),
            TokenKind::Punctuator(Punctuator::OpenParen | Punctuator::Colon)
        );
        let token = cursor.peek(0, interner).or_abrupt()?;
        match token.kind() {
            TokenKind::Keyword((Keyword::Async, true)) if is_keyword => {
                return Err(Error::general(
                    "Keyword must not contain escaped characters",
                    token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Async, false)) if is_keyword => {
                cursor.advance(interner);
                cursor.peek_expect_no_lineterminator(0, "Async object methods", interner)?;

                let token = cursor.peek(0, interner).or_abrupt()?;
                let position = token.span().start();

                if let TokenKind::Punctuator(Punctuator::Mul) = token.kind() {
                    let (class_element_name, method) =
                        AsyncGeneratorMethod::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;

                    // It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
                    if has_direct_super(&method) {
                        return Err(Error::general("invalid super usage", position));
                    }

                    let property_name =
                        if let property::ClassElementName::PropertyName(property_name) =
                            class_element_name
                        {
                            property_name
                        } else {
                            return Err(Error::general(
                                "private identifiers not allowed in object literal",
                                position,
                            ));
                        };

                    return Ok(property::PropertyDefinition::MethodDefinition(
                        property_name,
                        method,
                    ));
                }
                let (class_element_name, method) =
                    AsyncMethod::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

                let property::ClassElementName::PropertyName(property_name) = class_element_name else {
                    return Err(Error::general(
                        "private identifiers not allowed in object literal",
                        position,
                    ));
                };

                // It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
                if has_direct_super(&method) {
                    return Err(Error::general("invalid super usage", position));
                }

                return Ok(property::PropertyDefinition::MethodDefinition(
                    property_name,
                    method,
                ));
            }
            _ => {}
        }

        if cursor.peek(0, interner).or_abrupt()?.kind() == &TokenKind::Punctuator(Punctuator::Mul) {
            let position = cursor.peek(0, interner).or_abrupt()?.span().start();
            let (class_element_name, method) =
                GeneratorMethod::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

            // It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
            if has_direct_super(&method) {
                return Err(Error::general("invalid super usage", position));
            }

            match class_element_name {
                property::ClassElementName::PropertyName(property_name) => {
                    return Ok(property::PropertyDefinition::MethodDefinition(
                        property_name,
                        method,
                    ))
                }
                property::ClassElementName::PrivateIdentifier(_) => {
                    return Err(Error::general(
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
            return Ok(property::PropertyDefinition::Property(property_name, value));
        }

        let ordinary_method = cursor.peek(0, interner).or_abrupt()?.kind()
            == &TokenKind::Punctuator(Punctuator::OpenParen);

        match property_name {
            // MethodDefinition[?Yield, ?Await] -> get ClassElementName[?Yield, ?Await] ( ) { FunctionBody[~Yield, ~Await] }
            property::PropertyName::Literal(str) if str == Sym::GET && !ordinary_method => {
                let position = cursor.peek(0, interner).or_abrupt()?.span().start();

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
                let method = MethodDefinition::Get(Function::new(
                    None,
                    FormalParameterList::default(),
                    body,
                ));

                // It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
                if has_direct_super(&method) {
                    return Err(Error::general("invalid super usage", position));
                }

                Ok(property::PropertyDefinition::MethodDefinition(
                    property_name,
                    method,
                ))
            }
            // MethodDefinition[?Yield, ?Await] -> set ClassElementName[?Yield, ?Await] ( PropertySetParameterList ) { FunctionBody[~Yield, ~Await] }
            property::PropertyName::Literal(str) if str == Sym::SET && !ordinary_method => {
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
                    return Err(Error::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                let method = MethodDefinition::Set(Function::new(None, parameters, body));

                // It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
                if has_direct_super(&method) {
                    return Err(Error::general("invalid super usage", params_start_position));
                }

                Ok(property::PropertyDefinition::MethodDefinition(
                    property_name,
                    method,
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
                    return Err(Error::lex(LexError::Syntax(
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
                    return Err(Error::lex(LexError::Syntax(
                        "Illegal 'use strict' directive in function with non-simple parameter list"
                            .into(),
                        params_start_position,
                    )));
                }

                // It is a Syntax Error if any element of the BoundNames of FormalParameters also occurs in the
                // LexicallyDeclaredNames of FunctionBody.
                name_in_lexically_declared_names(
                    &bound_names(&params),
                    &top_level_lexically_declared_names(&body),
                    params_start_position,
                )?;

                let method = MethodDefinition::Ordinary(Function::new(None, params, body));

                // It is a Syntax Error if HasDirectSuper of MethodDefinition is true.
                if has_direct_super(&method) {
                    return Err(Error::general("invalid super usage", params_start_position));
                }

                Ok(property::PropertyDefinition::MethodDefinition(
                    property_name,
                    method,
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
pub(in crate::parser) struct PropertyName {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl PropertyName {
    /// Creates a new `PropertyName` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = property::PropertyName;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("PropertyName", "Parsing");

        let token = cursor.peek(0, interner).or_abrupt()?;
        let name = match token.kind() {
            TokenKind::Punctuator(Punctuator::OpenBracket) => {
                cursor.advance(interner);
                let node =
                    AssignmentExpression::new(None, false, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                cursor.expect(Punctuator::CloseBracket, "expected token ']'", interner)?;
                return Ok(node.into());
            }
            TokenKind::Identifier(name) | TokenKind::StringLiteral(name) => (*name).into(),
            TokenKind::NumericLiteral(num) => match num {
                Numeric::Rational(num) => Expression::Literal(Literal::from(*num)).into(),
                Numeric::Integer(num) => Expression::Literal(Literal::from(*num)).into(),
                Numeric::BigInt(num) => Expression::Literal(Literal::from(num.clone())).into(),
            },
            TokenKind::Keyword((word, _)) => {
                let (utf8, utf16) = word.as_str();
                interner.get_or_intern_static(utf8, utf16).into()
            }
            TokenKind::NullLiteral => (Sym::NULL).into(),
            TokenKind::BooleanLiteral(bool) => match bool {
                true => (interner.get_or_intern_static("true", utf16!("true"))).into(),
                false => (interner.get_or_intern_static("false", utf16!("false"))).into(),
            },
            _ => return Err(Error::AbruptEnd),
        };
        cursor.advance(interner);
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
pub(in crate::parser) struct ClassElementName {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassElementName {
    /// Creates a new `ClassElementName` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = property::ClassElementName;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("ClassElementName", "Parsing");

        match cursor.peek(0, interner).or_abrupt()?.kind() {
            TokenKind::PrivateIdentifier(ident) => {
                let ident = *ident;
                cursor.advance(interner);
                Ok(property::ClassElementName::PrivateIdentifier(ident))
            }
            _ => Ok(property::ClassElementName::PropertyName(
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
pub(in crate::parser) struct Initializer {
    name: Option<Identifier>,
    allow_in: AllowIn,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl Initializer {
    /// Creates a new `Initializer` parser.
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

impl<R> TokenParser<R> for Initializer
where
    R: Read,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
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
pub(in crate::parser) struct GeneratorMethod {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl GeneratorMethod {
    /// Creates a new `GeneratorMethod` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = (property::ClassElementName, MethodDefinition);

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("GeneratorMethod", "Parsing");
        cursor.expect(Punctuator::Mul, "generator method definition", interner)?;

        let class_element_name =
            ClassElementName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let params_start_position = cursor.peek(0, interner).or_abrupt()?.span().start();

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
            return Err(Error::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                body_start,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
        // occurs in the LexicallyDeclaredNames of GeneratorBody.
        name_in_lexically_declared_names(
            &bound_names(&params),
            &top_level_lexically_declared_names(&body),
            params_start_position,
        )?;

        let method = MethodDefinition::Generator(Generator::new(None, params, body, false));

        if contains(&method, ContainsSymbol::Super) {
            return Err(Error::lex(LexError::Syntax(
                "invalid super usage".into(),
                body_start,
            )));
        }

        Ok((class_element_name, method))
    }
}

/// `AsyncGeneratorMethod` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncGeneratorMethod
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct AsyncGeneratorMethod {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AsyncGeneratorMethod {
    /// Creates a new `AsyncGeneratorMethod` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = (property::ClassElementName, MethodDefinition);

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("AsyncGeneratorMethod", "Parsing");
        cursor.expect(
            Punctuator::Mul,
            "async generator method definition",
            interner,
        )?;

        let name =
            ClassElementName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let params_start_position = cursor.peek(0, interner).or_abrupt()?.span().start();

        let params = UniqueFormalParameters::new(true, true).parse(cursor, interner)?;

        // It is a Syntax Error if FormalParameters Contains YieldExpression is true.
        if contains(&params, ContainsSymbol::YieldExpression) {
            return Err(Error::lex(LexError::Syntax(
                "yield expression not allowed in async generator method definition parameters"
                    .into(),
                params_start_position,
            )));
        }

        // It is a Syntax Error if FormalParameters Contains AwaitExpression is true.
        if contains(&params, ContainsSymbol::AwaitExpression) {
            return Err(Error::lex(LexError::Syntax(
                "await expression not allowed in async generator method definition parameters"
                    .into(),
                params_start_position,
            )));
        }

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
            return Err(Error::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                body_start,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
        // occurs in the LexicallyDeclaredNames of GeneratorBody.
        name_in_lexically_declared_names(
            &bound_names(&params),
            &top_level_lexically_declared_names(&body),
            params_start_position,
        )?;

        let method =
            MethodDefinition::AsyncGenerator(AsyncGenerator::new(None, params, body, false));

        if contains(&method, ContainsSymbol::Super) {
            return Err(Error::lex(LexError::Syntax(
                "invalid super usage".into(),
                body_start,
            )));
        }

        Ok((name, method))
    }
}

/// `AsyncMethod` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-AsyncMethod
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct AsyncMethod {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl AsyncMethod {
    /// Creates a new `AsyncMethod` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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
    type Output = (property::ClassElementName, MethodDefinition);

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("AsyncMethod", "Parsing");

        let class_element_name =
            ClassElementName::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        let params_start_position = cursor.peek(0, interner).or_abrupt()?.span().start();

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
            return Err(Error::lex(LexError::Syntax(
                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                body_start,
            )));
        }

        // Early Error: It is a Syntax Error if any element of the BoundNames of UniqueFormalParameters also
        // occurs in the LexicallyDeclaredNames of GeneratorBody.
        name_in_lexically_declared_names(
            &bound_names(&params),
            &top_level_lexically_declared_names(&body),
            params_start_position,
        )?;

        let method = MethodDefinition::Async(AsyncFunction::new(None, params, body, false));

        if contains(&method, ContainsSymbol::Super) {
            return Err(Error::lex(LexError::Syntax(
                "invalid super usage".into(),
                body_start,
            )));
        }

        Ok((class_element_name, method))
    }
}

/// `CoverInitializedName` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CoverInitializedName
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct CoverInitializedName {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl CoverInitializedName {
    /// Creates a new `CoverInitializedName` parser.
    pub(in crate::parser) fn new<Y, A>(allow_yield: Y, allow_await: A) -> Self
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

impl<R> TokenParser<R> for CoverInitializedName
where
    R: Read,
{
    type Output = property::PropertyDefinition;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let _timer = Profiler::global().start_event("CoverInitializedName", "Parsing");

        let ident =
            IdentifierReference::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        cursor.expect(Punctuator::Assign, "CoverInitializedName", interner)?;

        let expr = AssignmentExpression::new(ident, true, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;

        Ok(property::PropertyDefinition::CoverInitializedName(
            ident, expr,
        ))
    }
}
