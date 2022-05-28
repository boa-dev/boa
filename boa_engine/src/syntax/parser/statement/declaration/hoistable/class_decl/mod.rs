use crate::syntax::{
    ast::{
        node::{
            self,
            declaration::class_decl::ClassElement as ClassElementNode,
            object::{MethodDefinition, PropertyName::Literal},
            Class, FormalParameterList, FunctionExpr,
        },
        Keyword, Punctuator,
    },
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::{
            AssignmentExpression, AsyncGeneratorMethod, AsyncMethod, BindingIdentifier,
            GeneratorMethod, LeftHandSideExpression, PropertyName,
        },
        function::{
            FormalParameter, FormalParameters, FunctionBody, UniqueFormalParameters,
            FUNCTION_BREAK_TOKENS,
        },
        statement::StatementList,
        AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use node::Node;
use rustc_hash::{FxHashMap, FxHashSet};
use std::io::Read;

/// Class declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
/// [spec]: https://tc39.es/ecma262/#prod-ClassDeclaration
#[derive(Debug, Clone, Copy)]
pub(super) struct ClassDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl ClassDeclaration {
    /// Creates a new `ClassDeclaration` parser.
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

impl<R> TokenParser<R> for ClassDeclaration
where
    R: Read,
{
    type Output = Node;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        cursor.expect((Keyword::Class, false), "class declaration", interner)?;
        let strict = cursor.strict_mode();
        cursor.set_strict_mode(true);

        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let name = match token.kind() {
            TokenKind::Identifier(_) | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
            }
            _ if self.is_default.0 => Sym::DEFAULT,
            _ => {
                return Err(ParseError::unexpected(
                    token.to_string(interner),
                    token.span(),
                    "expected class identifier",
                ))
            }
        };
        cursor.set_strict_mode(strict);

        Ok(Node::ClassDecl(
            ClassTail::new(name, self.allow_yield, self.allow_await).parse(cursor, interner)?,
        ))
    }
}

/// Class Tail parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassTail
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct ClassTail {
    name: Sym,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassTail {
    /// Creates a new `ClassTail` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(name: Sym, allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name,
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassTail
where
    R: Read,
{
    type Output = Class;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let super_ref = match token.kind() {
            TokenKind::Keyword((Keyword::Extends, true)) => {
                return Err(ParseError::general(
                    "Keyword must not contain escaped characters",
                    token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Extends, false)) => Some(Box::new(
                ClassHeritage::new(self.allow_yield, self.allow_await).parse(cursor, interner)?,
            )),
            _ => None,
        };

        cursor.expect(Punctuator::OpenBlock, "class tail", interner)?;

        // Temporarily disable strict mode because "strict" may be parsed as a keyword.
        let strict = cursor.strict_mode();
        cursor.set_strict_mode(false);
        let is_close_block = cursor
            .peek(0, interner)?
            .ok_or(ParseError::AbruptEnd)?
            .kind()
            == &TokenKind::Punctuator(Punctuator::CloseBlock);
        cursor.set_strict_mode(strict);

        if is_close_block {
            cursor.next(interner).expect("token disappeared");
            Ok(Class::new(self.name, super_ref, None, vec![]))
        } else {
            let (constructor, elements) =
                ClassBody::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
            cursor.expect(Punctuator::CloseBlock, "class tail", interner)?;
            Ok(Class::new(self.name, super_ref, constructor, elements))
        }
    }
}

/// `ClassHeritage` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassHeritage
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct ClassHeritage {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassHeritage {
    /// Creates a new `ClassHeritage` parser.
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

impl<R> TokenParser<R> for ClassHeritage
where
    R: Read,
{
    type Output = Node;

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        cursor.expect(
            TokenKind::Keyword((Keyword::Extends, false)),
            "class heritage",
            interner,
        )?;

        let strict = cursor.strict_mode();
        cursor.set_strict_mode(true);
        let lhs = LeftHandSideExpression::new(None, self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;
        cursor.set_strict_mode(strict);

        Ok(lhs)
    }
}

/// `ClassBody` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassBody
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct ClassBody {
    name: Sym,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassBody {
    /// Creates a new `ClassBody` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(name: Sym, allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name,
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassBody
where
    R: Read,
{
    type Output = (Option<FunctionExpr>, Vec<ClassElementNode>);

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        cursor.push_private_environment();

        let mut constructor = None;
        let mut elements = Vec::new();
        let mut private_elements_names = FxHashMap::default();

        // The identifier "static" is forbidden in strict mode but used as a keyword in classes.
        // Because of this, strict mode has to temporarily be disabled while parsing class field names.
        let strict = cursor.strict_mode();
        cursor.set_strict_mode(false);
        loop {
            let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
            let position = token.span().start();
            match token.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock) => break,
                _ => match ClassElement::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
                {
                    (Some(_), None) if constructor.is_some() => {
                        return Err(ParseError::general(
                            "a class may only have one constructor",
                            position,
                        ));
                    }
                    (Some(c), None) => {
                        constructor = Some(c);
                    }
                    (None, Some(element)) => {
                        match &element {
                            ClassElementNode::PrivateMethodDefinition(name, method) => match method
                            {
                                MethodDefinition::Get(_) => {
                                    match private_elements_names.get(name) {
                                        Some(PrivateElement::Setter) => {
                                            private_elements_names
                                                .insert(*name, PrivateElement::Value);
                                        }
                                        Some(_) => {
                                            return Err(ParseError::general(
                                                "private identifier has already been declared",
                                                position,
                                            ));
                                        }
                                        None => {
                                            private_elements_names
                                                .insert(*name, PrivateElement::Getter);
                                        }
                                    }
                                }
                                MethodDefinition::Set(_) => {
                                    match private_elements_names.get(name) {
                                        Some(PrivateElement::Getter) => {
                                            private_elements_names
                                                .insert(*name, PrivateElement::Value);
                                        }
                                        Some(_) => {
                                            return Err(ParseError::general(
                                                "private identifier has already been declared",
                                                position,
                                            ));
                                        }
                                        None => {
                                            private_elements_names
                                                .insert(*name, PrivateElement::Setter);
                                        }
                                    }
                                }
                                _ => {
                                    if private_elements_names
                                        .insert(*name, PrivateElement::Value)
                                        .is_some()
                                    {
                                        return Err(ParseError::general(
                                            "private identifier has already been declared",
                                            position,
                                        ));
                                    }
                                }
                            },
                            ClassElementNode::PrivateStaticMethodDefinition(name, method) => {
                                match method {
                                    MethodDefinition::Get(_) => {
                                        match private_elements_names.get(name) {
                                            Some(PrivateElement::StaticSetter) => {
                                                private_elements_names
                                                    .insert(*name, PrivateElement::StaticValue);
                                            }
                                            Some(_) => {
                                                return Err(ParseError::general(
                                                    "private identifier has already been declared",
                                                    position,
                                                ));
                                            }
                                            None => {
                                                private_elements_names
                                                    .insert(*name, PrivateElement::StaticGetter);
                                            }
                                        }
                                    }
                                    MethodDefinition::Set(_) => {
                                        match private_elements_names.get(name) {
                                            Some(PrivateElement::StaticGetter) => {
                                                private_elements_names
                                                    .insert(*name, PrivateElement::StaticValue);
                                            }
                                            Some(_) => {
                                                return Err(ParseError::general(
                                                    "private identifier has already been declared",
                                                    position,
                                                ));
                                            }
                                            None => {
                                                private_elements_names
                                                    .insert(*name, PrivateElement::StaticSetter);
                                            }
                                        }
                                    }
                                    _ => {
                                        if private_elements_names
                                            .insert(*name, PrivateElement::StaticValue)
                                            .is_some()
                                        {
                                            return Err(ParseError::general(
                                                "private identifier has already been declared",
                                                position,
                                            ));
                                        }
                                    }
                                }
                            }
                            ClassElementNode::PrivateFieldDefinition(name, _) => {
                                if private_elements_names
                                    .insert(*name, PrivateElement::Value)
                                    .is_some()
                                {
                                    return Err(ParseError::general(
                                        "private identifier has already been declared",
                                        position,
                                    ));
                                }
                            }
                            ClassElementNode::PrivateStaticFieldDefinition(name, _) => {
                                if private_elements_names
                                    .insert(*name, PrivateElement::StaticValue)
                                    .is_some()
                                {
                                    return Err(ParseError::general(
                                        "private identifier has already been declared",
                                        position,
                                    ));
                                }
                            }
                            _ => {}
                        }
                        elements.push(element);
                    }
                    _ => {}
                },
            }
        }

        cursor.set_strict_mode(strict);
        cursor.pop_private_environment(&private_elements_names)?;

        Ok((constructor, elements))
    }
}

/// Representation of private object elements.
#[derive(Debug, PartialEq)]
pub(in crate::syntax) enum PrivateElement {
    Value,
    Getter,
    Setter,
    StaticValue,
    StaticSetter,
    StaticGetter,
}

/// `ClassElement` parsing.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElement
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct ClassElement {
    name: Sym,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassElement {
    /// Creates a new `ClassElement` parser.
    pub(in crate::syntax::parser) fn new<Y, A>(name: Sym, allow_yield: Y, allow_await: A) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name,
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassElement
where
    R: Read,
{
    type Output = (Option<FunctionExpr>, Option<ClassElementNode>);

    fn parse(
        self,
        cursor: &mut Cursor<R>,
        interner: &mut Interner,
    ) -> Result<Self::Output, ParseError> {
        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let r#static = match token.kind() {
            TokenKind::Punctuator(Punctuator::Semicolon) => {
                cursor.next(interner).expect("token disappeared");
                return Ok((None, None));
            }
            TokenKind::Identifier(Sym::STATIC) => {
                let token = cursor.peek(1, interner)?.ok_or(ParseError::AbruptEnd)?;
                match token.kind() {
                    TokenKind::Identifier(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::NumericLiteral(_)
                    | TokenKind::Keyword(_)
                    | TokenKind::NullLiteral
                    | TokenKind::PrivateIdentifier(_)
                    | TokenKind::Punctuator(
                        Punctuator::OpenBracket
                        | Punctuator::Mul
                        | Punctuator::OpenBlock
                        | Punctuator::Semicolon,
                    ) => {
                        cursor.next(interner).expect("token disappeared");
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        };

        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let position = token.span().start();
        let element = match token.kind() {
            TokenKind::Identifier(Sym::CONSTRUCTOR) if !r#static => {
                cursor.next(interner).expect("token disappeared");
                let strict = cursor.strict_mode();
                cursor.set_strict_mode(true);

                cursor.expect(Punctuator::OpenParen, "class constructor", interner)?;
                let parameters = FormalParameters::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                cursor.expect(Punctuator::CloseParen, "class constructor", interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "class constructor",
                    interner,
                )?;
                let body = FunctionBody::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "class constructor",
                    interner,
                )?;
                cursor.set_strict_mode(strict);

                return Ok((Some(FunctionExpr::new(self.name, parameters, body)), None));
            }
            TokenKind::Punctuator(Punctuator::Semicolon) if r#static => {
                cursor.next(interner).expect("token disappeared");
                ClassElementNode::FieldDefinition(Sym::STATIC.into(), None)
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) if r#static => {
                cursor.next(interner).expect("token disappeared");
                let statement_list = if cursor
                    .next_if(TokenKind::Punctuator(Punctuator::CloseBlock), interner)?
                    .is_some()
                {
                    node::StatementList::from(vec![])
                } else {
                    let strict = cursor.strict_mode();
                    cursor.set_strict_mode(true);
                    let position = cursor
                        .peek(0, interner)?
                        .ok_or(ParseError::AbruptEnd)?
                        .span()
                        .start();
                    let statement_list =
                        StatementList::new(false, true, false, &FUNCTION_BREAK_TOKENS)
                            .parse(cursor, interner)?;

                    let lexically_declared_names = statement_list.lexically_declared_names();
                    let mut lexically_declared_names_map: FxHashMap<Sym, bool> =
                        FxHashMap::default();
                    for (name, is_function_declaration) in &lexically_declared_names {
                        if let Some(existing_is_function_declaration) =
                            lexically_declared_names_map.get(name)
                        {
                            if !(!cursor.strict_mode()
                                && *is_function_declaration
                                && *existing_is_function_declaration)
                            {
                                return Err(ParseError::general(
                                    "lexical name declared multiple times",
                                    position,
                                ));
                            }
                        }
                        lexically_declared_names_map.insert(*name, *is_function_declaration);
                    }

                    let mut var_declared_names = FxHashSet::default();
                    statement_list.var_declared_names_new(&mut var_declared_names);
                    for (lex_name, _) in &lexically_declared_names {
                        if var_declared_names.contains(lex_name) {
                            return Err(ParseError::general(
                                "lexical name declared in var names",
                                position,
                            ));
                        }
                    }

                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "class definition",
                        interner,
                    )?;
                    cursor.set_strict_mode(strict);
                    statement_list
                };
                ClassElementNode::StaticBlock(statement_list)
            }
            TokenKind::Punctuator(Punctuator::Mul) => {
                let token = cursor.peek(1, interner)?.ok_or(ParseError::AbruptEnd)?;
                let name_position = token.span().start();
                if let TokenKind::Identifier(Sym::CONSTRUCTOR) = token.kind() {
                    return Err(ParseError::general(
                        "class constructor may not be a generator method",
                        token.span().start(),
                    ));
                }
                let strict = cursor.strict_mode();
                cursor.set_strict_mode(true);
                let (class_element_name, method) =
                    GeneratorMethod::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                cursor.set_strict_mode(strict);

                match class_element_name {
                    node::object::ClassElementName::PropertyName(property_name) if r#static => {
                        if let Some(Sym::PROTOTYPE) = property_name.prop_name() {
                            return Err(ParseError::general(
                                "class may not have static method definitions named 'prototype'",
                                name_position,
                            ));
                        }
                        ClassElementNode::StaticMethodDefinition(property_name, method)
                    }
                    node::object::ClassElementName::PropertyName(property_name) => {
                        ClassElementNode::MethodDefinition(property_name, method)
                    }
                    node::object::ClassElementName::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                        return Err(ParseError::general(
                            "class constructor may not be a private method",
                            name_position,
                        ))
                    }
                    node::object::ClassElementName::PrivateIdentifier(private_ident)
                        if r#static =>
                    {
                        ClassElementNode::PrivateStaticMethodDefinition(private_ident, method)
                    }
                    node::object::ClassElementName::PrivateIdentifier(private_ident) => {
                        ClassElementNode::PrivateMethodDefinition(private_ident, method)
                    }
                }
            }
            TokenKind::Keyword((Keyword::Async, true)) => {
                return Err(ParseError::general(
                    "Keyword must not contain escaped characters",
                    token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Async, false)) => {
                cursor.next(interner).expect("token disappeared");
                cursor.peek_expect_no_lineterminator(0, "Async object methods", interner)?;
                let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Mul) => {
                        let token = cursor.peek(1, interner)?.ok_or(ParseError::AbruptEnd)?;
                        let name_position = token.span().start();
                        if let TokenKind::Identifier(Sym::CONSTRUCTOR) = token.kind() {
                            return Err(ParseError::general(
                                "class constructor may not be a generator method",
                                token.span().start(),
                            ));
                        }
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let (property_name, method) =
                            AsyncGeneratorMethod::new(self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?;
                        cursor.set_strict_mode(strict);
                        if r#static {
                            if let Some(name) = property_name.prop_name() {
                                if name == Sym::PROTOTYPE {
                                    return Err(ParseError::general(
                                            "class may not have static method definitions named 'prototype'",
                                            name_position,
                                        ));
                                }
                            }
                            ClassElementNode::StaticMethodDefinition(property_name, method)
                        } else {
                            ClassElementNode::MethodDefinition(property_name, method)
                        }
                    }
                    TokenKind::Identifier(Sym::CONSTRUCTOR) => {
                        return Err(ParseError::general(
                            "class constructor may not be an async method",
                            token.span().start(),
                        ))
                    }
                    _ => {
                        let name_position = token.span().start();
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let (property_name, method) =
                            AsyncMethod::new(self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?;
                        cursor.set_strict_mode(strict);
                        if r#static {
                            if let Some(name) = property_name.prop_name() {
                                if name == Sym::PROTOTYPE {
                                    return Err(ParseError::general(
                                            "class may not have static method definitions named 'prototype'",
                                            name_position,
                                        ));
                                }
                            }
                            ClassElementNode::StaticMethodDefinition(property_name, method)
                        } else {
                            ClassElementNode::MethodDefinition(property_name, method)
                        }
                    }
                }
            }
            TokenKind::Identifier(Sym::GET) => {
                cursor.next(interner).expect("token disappeared");
                let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Assign) => {
                        cursor.next(interner).expect("token disappeared");
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let rhs = AssignmentExpression::new(
                            Sym::GET,
                            true,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?;
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        cursor.set_strict_mode(strict);
                        if r#static {
                            ClassElementNode::StaticFieldDefinition(Literal(Sym::GET), Some(rhs))
                        } else {
                            ClassElementNode::FieldDefinition(Literal(Sym::GET), Some(rhs))
                        }
                    }
                    TokenKind::Punctuator(Punctuator::OpenParen) => {
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let params =
                            UniqueFormalParameters::new(false, false).parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "method definition",
                            interner,
                        )?;
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        let token = cursor.expect(
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
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict_mode(strict);
                        let method =
                            MethodDefinition::Ordinary(FunctionExpr::new(None, params, body));
                        if r#static {
                            ClassElementNode::StaticMethodDefinition(Literal(Sym::GET), method)
                        } else {
                            ClassElementNode::MethodDefinition(Literal(Sym::GET), method)
                        }
                    }
                    TokenKind::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                        return Err(ParseError::general(
                            "class constructor may not be a private method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::PrivateIdentifier(name) => {
                        let name = *name;
                        cursor.next(interner).expect("token disappeared");
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let params =
                            UniqueFormalParameters::new(false, false).parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "method definition",
                            interner,
                        )?;
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        let token = cursor.expect(
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
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict_mode(strict);
                        let method = MethodDefinition::Get(FunctionExpr::new(None, params, body));
                        if r#static {
                            ClassElementNode::PrivateStaticMethodDefinition(name, method)
                        } else {
                            ClassElementNode::PrivateMethodDefinition(name, method)
                        }
                    }
                    TokenKind::Identifier(Sym::CONSTRUCTOR) => {
                        return Err(ParseError::general(
                            "class constructor may not be a getter method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::Identifier(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::NumericLiteral(_)
                    | TokenKind::Keyword(_)
                    | TokenKind::NullLiteral
                    | TokenKind::Punctuator(Punctuator::OpenBracket) => {
                        let name_position = token.span().start();
                        let name = PropertyName::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenParen),
                            "class getter",
                            interner,
                        )?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::CloseParen),
                            "class getter",
                            interner,
                        )?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "class getter",
                            interner,
                        )?;
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        cursor.set_strict_mode(strict);
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::CloseBlock),
                            "class getter",
                            interner,
                        )?;

                        let method = MethodDefinition::Get(FunctionExpr::new(
                            None,
                            FormalParameterList::empty(),
                            body,
                        ));
                        if r#static {
                            if let Some(name) = name.prop_name() {
                                if name == Sym::PROTOTYPE {
                                    return Err(ParseError::general(
                                            "class may not have static method definitions named 'prototype'",
                                            name_position,
                                        ));
                                }
                            }
                            ClassElementNode::StaticMethodDefinition(name, method)
                        } else {
                            ClassElementNode::MethodDefinition(name, method)
                        }
                    }
                    _ => {
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            ClassElementNode::StaticFieldDefinition(Literal(Sym::GET), None)
                        } else {
                            ClassElementNode::FieldDefinition(Literal(Sym::GET), None)
                        }
                    }
                }
            }
            TokenKind::Identifier(Sym::SET) => {
                cursor.next(interner).expect("token disappeared");
                let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Assign) => {
                        cursor.next(interner).expect("token disappeared");
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let rhs = AssignmentExpression::new(
                            Sym::SET,
                            true,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?;
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        cursor.set_strict_mode(strict);
                        if r#static {
                            ClassElementNode::StaticFieldDefinition(Literal(Sym::SET), Some(rhs))
                        } else {
                            ClassElementNode::FieldDefinition(Literal(Sym::SET), Some(rhs))
                        }
                    }
                    TokenKind::Punctuator(Punctuator::OpenParen) => {
                        cursor.next(interner).expect("token disappeared");
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let parameters: FormalParameterList = FormalParameter::new(false, false)
                            .parse(cursor, interner)?
                            .into();
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::CloseParen),
                            "class setter method definition",
                            interner,
                        )?;

                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "method definition",
                            interner,
                        )?;
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        let token = cursor.expect(
                            TokenKind::Punctuator(Punctuator::CloseBlock),
                            "method definition",
                            interner,
                        )?;

                        // Early Error: It is a Syntax Error if FunctionBodyContainsUseStrict of FunctionBody is true
                        // and IsSimpleParameterList of UniqueFormalParameters is false.
                        if body.strict() && !parameters.is_simple() {
                            return Err(ParseError::lex(LexError::Syntax(
                            "Illegal 'use strict' directive in function with non-simple parameter list"
                                .into(),
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict_mode(strict);
                        let method =
                            MethodDefinition::Ordinary(FunctionExpr::new(None, parameters, body));
                        if r#static {
                            ClassElementNode::StaticMethodDefinition(Literal(Sym::SET), method)
                        } else {
                            ClassElementNode::MethodDefinition(Literal(Sym::SET), method)
                        }
                    }
                    TokenKind::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                        return Err(ParseError::general(
                            "class constructor may not be a private method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::PrivateIdentifier(name) => {
                        let name = *name;
                        cursor.next(interner).expect("token disappeared");
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let params =
                            UniqueFormalParameters::new(false, false).parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "method definition",
                            interner,
                        )?;
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        let token = cursor.expect(
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
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict_mode(strict);
                        let method = MethodDefinition::Set(FunctionExpr::new(None, params, body));
                        if r#static {
                            ClassElementNode::PrivateStaticMethodDefinition(name, method)
                        } else {
                            ClassElementNode::PrivateMethodDefinition(name, method)
                        }
                    }
                    TokenKind::Identifier(Sym::CONSTRUCTOR) => {
                        return Err(ParseError::general(
                            "class constructor may not be a setter method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::Identifier(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::NumericLiteral(_)
                    | TokenKind::Keyword(_)
                    | TokenKind::NullLiteral
                    | TokenKind::Punctuator(Punctuator::OpenBracket) => {
                        let name_position = token.span().start();
                        let name = PropertyName::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let params =
                            UniqueFormalParameters::new(false, false).parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "method definition",
                            interner,
                        )?;
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        let token = cursor.expect(
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
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict_mode(strict);
                        let method = MethodDefinition::Set(FunctionExpr::new(None, params, body));
                        if r#static {
                            if let Some(name) = name.prop_name() {
                                if name == Sym::PROTOTYPE {
                                    return Err(ParseError::general(
                                            "class may not have static method definitions named 'prototype'",
                                            name_position,
                                        ));
                                }
                            }
                            ClassElementNode::StaticMethodDefinition(name, method)
                        } else {
                            ClassElementNode::MethodDefinition(name, method)
                        }
                    }
                    _ => {
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            ClassElementNode::StaticFieldDefinition(Literal(Sym::SET), None)
                        } else {
                            ClassElementNode::FieldDefinition(Literal(Sym::SET), None)
                        }
                    }
                }
            }
            TokenKind::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                return Err(ParseError::general(
                    "class constructor may not be a private method",
                    token.span().start(),
                ))
            }
            TokenKind::PrivateIdentifier(name) => {
                let name = *name;
                cursor.next(interner).expect("token disappeared");
                let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Assign) => {
                        cursor.next(interner).expect("token disappeared");
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let rhs = AssignmentExpression::new(
                            name,
                            true,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?;
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        cursor.set_strict_mode(strict);
                        if r#static {
                            ClassElementNode::PrivateStaticFieldDefinition(name, Some(rhs))
                        } else {
                            ClassElementNode::PrivateFieldDefinition(name, Some(rhs))
                        }
                    }
                    TokenKind::Punctuator(Punctuator::OpenParen) => {
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let params =
                            UniqueFormalParameters::new(false, false).parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "method definition",
                            interner,
                        )?;
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        let token = cursor.expect(
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
                                token.span().start(),
                        )));
                        }
                        let method =
                            MethodDefinition::Ordinary(FunctionExpr::new(None, params, body));
                        cursor.set_strict_mode(strict);
                        if r#static {
                            ClassElementNode::PrivateStaticMethodDefinition(name, method)
                        } else {
                            ClassElementNode::PrivateMethodDefinition(name, method)
                        }
                    }
                    _ => {
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            ClassElementNode::PrivateStaticFieldDefinition(name, None)
                        } else {
                            ClassElementNode::PrivateFieldDefinition(name, None)
                        }
                    }
                }
            }
            TokenKind::Identifier(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::NumericLiteral(_)
            | TokenKind::Keyword(_)
            | TokenKind::NullLiteral
            | TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let name_position = token.span().start();
                let name = PropertyName::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Assign) => {
                        if let Some(name) = name.prop_name() {
                            if r#static {
                                if [Sym::CONSTRUCTOR, Sym::PROTOTYPE].contains(&name) {
                                    return Err(ParseError::general(
                                        "class may not have static field definitions named 'constructor' or 'prototype'",
                                        name_position,
                                    ));
                                }
                            } else if name == Sym::CONSTRUCTOR {
                                return Err(ParseError::general(
                                    "class may not have field definitions named 'constructor'",
                                    name_position,
                                ));
                            }
                        }
                        cursor.next(interner).expect("token disappeared");
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let rhs = AssignmentExpression::new(
                            name.literal(),
                            true,
                            self.allow_yield,
                            self.allow_await,
                        )
                        .parse(cursor, interner)?;
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        cursor.set_strict_mode(strict);
                        if r#static {
                            ClassElementNode::StaticFieldDefinition(name, Some(rhs))
                        } else {
                            ClassElementNode::FieldDefinition(name, Some(rhs))
                        }
                    }
                    TokenKind::Punctuator(Punctuator::OpenParen) => {
                        if let Some(name) = name.prop_name() {
                            if r#static && name == Sym::PROTOTYPE {
                                return Err(ParseError::general(
                                        "class may not have static method definitions named 'prototype'",
                                        name_position,
                                    ));
                            }
                        }
                        let strict = cursor.strict_mode();
                        cursor.set_strict_mode(true);
                        let params =
                            UniqueFormalParameters::new(false, false).parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::OpenBlock),
                            "method definition",
                            interner,
                        )?;
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        let token = cursor.expect(
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
                                token.span().start(),
                        )));
                        }
                        let method =
                            MethodDefinition::Ordinary(FunctionExpr::new(None, params, body));
                        cursor.set_strict_mode(strict);
                        if r#static {
                            ClassElementNode::StaticMethodDefinition(name, method)
                        } else {
                            ClassElementNode::MethodDefinition(name, method)
                        }
                    }
                    _ => {
                        if let Some(name) = name.prop_name() {
                            if r#static {
                                if [Sym::CONSTRUCTOR, Sym::PROTOTYPE].contains(&name) {
                                    return Err(ParseError::general(
                                        "class may not have static field definitions named 'constructor' or 'prototype'",
                                        name_position,
                                    ));
                                }
                            } else if name == Sym::CONSTRUCTOR {
                                return Err(ParseError::general(
                                    "class may not have field definitions named 'constructor'",
                                    name_position,
                                ));
                            }
                        }
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            ClassElementNode::StaticFieldDefinition(name, None)
                        } else {
                            ClassElementNode::FieldDefinition(name, None)
                        }
                    }
                }
            }
            _ => {
                return Err(ParseError::general(
                    "unexpected token",
                    token.span().start(),
                ))
            }
        };

        match &element {
            // FieldDefinition : ClassElementName Initializer [opt]
            // It is a Syntax Error if Initializer is present and ContainsArguments of Initializer is true.
            ClassElementNode::FieldDefinition(_, Some(node))
            | ClassElementNode::StaticFieldDefinition(_, Some(node))
            | ClassElementNode::PrivateFieldDefinition(_, Some(node))
            | ClassElementNode::PrivateStaticFieldDefinition(_, Some(node)) => {
                if node.contains_arguments() {
                    return Err(ParseError::general(
                        "'arguments' not allowed in class field definition",
                        position,
                    ));
                }
            }
            // ClassStaticBlockBody : ClassStaticBlockStatementList
            // It is a Syntax Error if ContainsArguments of ClassStaticBlockStatementList is true.
            ClassElementNode::StaticBlock(block) => {
                for node in block.items() {
                    if node.contains_arguments() {
                        return Err(ParseError::general(
                            "'arguments' not allowed in class static block",
                            position,
                        ));
                    }
                }
            }
            _ => {}
        }

        Ok((None, Some(element)))
    }
}
