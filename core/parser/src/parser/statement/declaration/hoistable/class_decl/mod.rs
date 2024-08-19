#[cfg(test)]
mod tests;

use crate::{
    lexer::{token::ContainsEscapeSequence, Error as LexError, TokenKind},
    parser::{
        expression::{
            AssignmentExpression, AsyncGeneratorMethod, AsyncMethod, BindingIdentifier,
            GeneratorMethod, LeftHandSideExpression, PropertyName,
        },
        function::{FunctionBody, UniqueFormalParameters, FUNCTION_BREAK_TOKENS},
        statement::StatementList,
        AllowAwait, AllowDefault, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
    },
    source::ReadChar,
    Error,
};
use ast::{
    function::PrivateName,
    operations::{
        check_labels, contains_invalid_object_literal, lexically_declared_names, var_declared_names,
    },
    property::MethodDefinitionKind,
};
use boa_ast::{
    self as ast,
    expression::Identifier,
    function::{
        self, ClassDeclaration as ClassDeclarationNode, ClassElementName, ClassMethodDefinition,
        FormalParameterList, FunctionExpression,
    },
    operations::{contains, contains_arguments, ContainsSymbol},
    Expression, Keyword, Punctuator,
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;
use rustc_hash::{FxHashMap, FxHashSet};

/// Class declaration parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
/// [spec]: https://tc39.es/ecma262/#prod-ClassDeclaration
#[derive(Debug, Clone, Copy)]
pub(in crate::parser) struct ClassDeclaration {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    is_default: AllowDefault,
}

impl ClassDeclaration {
    /// Creates a new `ClassDeclaration` parser.
    pub(in crate::parser) fn new<Y, A, D>(allow_yield: Y, allow_await: A, is_default: D) -> Self
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
    R: ReadChar,
{
    type Output = ClassDeclarationNode;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect((Keyword::Class, false), "class declaration", interner)?;
        let strict = cursor.strict();
        cursor.set_strict(true);

        let token = cursor.peek(0, interner).or_abrupt()?;
        let name = match token.kind() {
            TokenKind::IdentifierName(_)
            | TokenKind::Keyword((Keyword::Yield | Keyword::Await, _)) => {
                BindingIdentifier::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?
            }
            _ if self.is_default.0 => Sym::DEFAULT.into(),
            _ => {
                return Err(Error::unexpected(
                    token.to_string(interner),
                    token.span(),
                    "expected class identifier",
                ))
            }
        };
        cursor.set_strict(strict);

        let (super_ref, constructor, elements) =
            ClassTail::new(name, self.allow_yield, self.allow_await).parse(cursor, interner)?;

        Ok(ClassDeclarationNode::new(
            name,
            super_ref,
            constructor,
            elements.into_boxed_slice(),
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
pub(in crate::parser) struct ClassTail {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassTail {
    /// Creates a new `ClassTail` parser.
    pub(in crate::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassTail
where
    R: ReadChar,
{
    type Output = (
        Option<Expression>,
        Option<FunctionExpression>,
        Vec<function::ClassElement>,
    );

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let token = cursor.peek(0, interner).or_abrupt()?;
        let super_ref = match token.kind() {
            TokenKind::Keyword((Keyword::Extends, true)) => {
                return Err(Error::general(
                    "Keyword must not contain escaped characters",
                    token.span().start(),
                ));
            }
            TokenKind::Keyword((Keyword::Extends, false)) => Some(
                ClassHeritage::new(self.allow_yield, self.allow_await).parse(cursor, interner)?,
            ),
            _ => None,
        };

        cursor.expect(Punctuator::OpenBlock, "class tail", interner)?;

        // Temporarily disable strict mode because "strict" may be parsed as a keyword.
        let strict = cursor.strict();
        cursor.set_strict(false);
        let is_close_block = cursor.peek(0, interner).or_abrupt()?.kind()
            == &TokenKind::Punctuator(Punctuator::CloseBlock);
        cursor.set_strict(strict);

        if is_close_block {
            cursor.advance(interner);
            Ok((super_ref, None, Vec::new()))
        } else {
            let body_start = cursor.peek(0, interner).or_abrupt()?.span().start();
            let (constructor, elements) =
                ClassBody::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
            cursor.expect(Punctuator::CloseBlock, "class tail", interner)?;

            if super_ref.is_none() {
                if let Some(constructor) = &constructor {
                    if contains(constructor, ContainsSymbol::SuperCall) {
                        return Err(Error::lex(LexError::Syntax(
                            "invalid super usage".into(),
                            body_start,
                        )));
                    }
                }
            }

            Ok((super_ref, constructor, elements))
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
pub(in crate::parser) struct ClassHeritage {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassHeritage {
    /// Creates a new `ClassHeritage` parser.
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

impl<R> TokenParser<R> for ClassHeritage
where
    R: ReadChar,
{
    type Output = Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        cursor.expect(
            TokenKind::Keyword((Keyword::Extends, false)),
            "class heritage",
            interner,
        )?;

        let strict = cursor.strict();
        cursor.set_strict(true);
        let lhs = LeftHandSideExpression::new(self.allow_yield, self.allow_await)
            .parse(cursor, interner)?;
        cursor.set_strict(strict);

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
pub(in crate::parser) struct ClassBody {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassBody {
    /// Creates a new `ClassBody` parser.
    pub(in crate::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassBody
where
    R: ReadChar,
{
    type Output = (Option<FunctionExpression>, Vec<function::ClassElement>);

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let mut constructor = None;
        let mut elements = Vec::new();
        let mut private_elements_names = FxHashMap::default();

        // The identifier "static" is forbidden in strict mode but used as a keyword in classes.
        // Because of this, strict mode has to temporarily be disabled while parsing class field names.
        let strict = cursor.strict();
        cursor.set_strict(false);
        loop {
            let token = cursor.peek(0, interner).or_abrupt()?;
            let position = token.span().start();
            let (parsed_constructor, element) = match token.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock) => break,
                _ => ClassElement::new(self.name, self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?,
            };
            if let Some(c) = parsed_constructor {
                if constructor.is_some() {
                    return Err(Error::general(
                        "a class may only have one constructor",
                        position,
                    ));
                }
                constructor = Some(c);
            }
            let Some(element) = element else {
                continue;
            };

            match &element {
                function::ClassElement::MethodDefinition(m) => {
                    // It is a Syntax Error if PropName of MethodDefinition is not "constructor" and HasDirectSuper of MethodDefinition is true.
                    if let ClassElementName::PropertyName(name) = m.name() {
                        if contains(name, ContainsSymbol::SuperCall) {
                            return Err(Error::lex(LexError::Syntax(
                                "invalid super call usage".into(),
                                position,
                            )));
                        }
                    }
                    if contains(m.parameters(), ContainsSymbol::SuperCall)
                        || contains(m.body(), ContainsSymbol::SuperCall)
                    {
                        return Err(Error::lex(LexError::Syntax(
                            "invalid super call usage".into(),
                            position,
                        )));
                    }

                    if let ClassElementName::PrivateName(name) = m.name() {
                        match m.kind() {
                            MethodDefinitionKind::Get => {
                                match private_elements_names.get(&name.description()) {
                                    Some(PrivateElement::StaticSetter) if m.is_static() => {
                                        private_elements_names.insert(
                                            name.description(),
                                            PrivateElement::StaticValue,
                                        );
                                    }
                                    Some(PrivateElement::Setter) if !m.is_static() => {
                                        private_elements_names
                                            .insert(name.description(), PrivateElement::Value);
                                    }
                                    Some(_) => {
                                        return Err(Error::general(
                                            "private identifier has already been declared",
                                            position,
                                        ));
                                    }
                                    None => {
                                        private_elements_names.insert(
                                            name.description(),
                                            if m.is_static() {
                                                PrivateElement::StaticGetter
                                            } else {
                                                PrivateElement::Getter
                                            },
                                        );
                                    }
                                }
                            }
                            MethodDefinitionKind::Set => {
                                match private_elements_names.get(&name.description()) {
                                    Some(PrivateElement::StaticGetter) if m.is_static() => {
                                        private_elements_names.insert(
                                            name.description(),
                                            PrivateElement::StaticValue,
                                        );
                                    }
                                    Some(PrivateElement::Getter) if !m.is_static() => {
                                        private_elements_names
                                            .insert(name.description(), PrivateElement::Value);
                                    }
                                    Some(_) => {
                                        return Err(Error::general(
                                            "private identifier has already been declared",
                                            position,
                                        ));
                                    }
                                    None => {
                                        private_elements_names.insert(
                                            name.description(),
                                            if m.is_static() {
                                                PrivateElement::StaticSetter
                                            } else {
                                                PrivateElement::Setter
                                            },
                                        );
                                    }
                                }
                            }
                            _ => {
                                if private_elements_names
                                    .insert(
                                        name.description(),
                                        if m.is_static() {
                                            PrivateElement::StaticValue
                                        } else {
                                            PrivateElement::Value
                                        },
                                    )
                                    .is_some()
                                {
                                    return Err(Error::general(
                                        "private identifier has already been declared",
                                        position,
                                    ));
                                }
                            }
                        }
                    }
                }
                function::ClassElement::PrivateFieldDefinition(name, init) => {
                    if let Some(node) = init {
                        if contains(node, ContainsSymbol::SuperCall) {
                            return Err(Error::lex(LexError::Syntax(
                                "invalid super usage".into(),
                                position,
                            )));
                        }
                    }
                    if private_elements_names
                        .insert(name.description(), PrivateElement::Value)
                        .is_some()
                    {
                        return Err(Error::general(
                            "private identifier has already been declared",
                            position,
                        ));
                    }
                }
                function::ClassElement::PrivateStaticFieldDefinition(name, init) => {
                    if let Some(node) = init {
                        if contains(node, ContainsSymbol::SuperCall) {
                            return Err(Error::lex(LexError::Syntax(
                                "invalid super usage".into(),
                                position,
                            )));
                        }
                    }
                    if private_elements_names
                        .insert(name.description(), PrivateElement::StaticValue)
                        .is_some()
                    {
                        return Err(Error::general(
                            "private identifier has already been declared",
                            position,
                        ));
                    }
                }
                function::ClassElement::FieldDefinition(_, Some(node))
                | function::ClassElement::StaticFieldDefinition(_, Some(node)) => {
                    if contains(node, ContainsSymbol::SuperCall) {
                        return Err(Error::lex(LexError::Syntax(
                            "invalid super usage".into(),
                            position,
                        )));
                    }
                }
                _ => {}
            }
            elements.push(element);
        }

        cursor.set_strict(strict);

        Ok((constructor, elements))
    }
}

/// Representation of private object elements.
#[derive(Debug, PartialEq)]
pub(crate) enum PrivateElement {
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
pub(in crate::parser) struct ClassElement {
    name: Option<Identifier>,
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassElement {
    /// Creates a new `ClassElement` parser.
    pub(in crate::parser) fn new<N, Y, A>(name: N, allow_yield: Y, allow_await: A) -> Self
    where
        N: Into<Option<Identifier>>,
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            name: name.into(),
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
        }
    }
}

impl<R> TokenParser<R> for ClassElement
where
    R: ReadChar,
{
    type Output = (Option<FunctionExpression>, Option<function::ClassElement>);

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let token = cursor.peek(0, interner).or_abrupt()?;
        let r#static = match token.kind() {
            TokenKind::Punctuator(Punctuator::Semicolon) => {
                cursor.advance(interner);
                return Ok((None, None));
            }
            TokenKind::IdentifierName((Sym::STATIC, ContainsEscapeSequence(contains_escape))) => {
                let contains_escape = *contains_escape;
                let token = cursor.peek(1, interner).or_abrupt()?;
                match token.kind() {
                    TokenKind::IdentifierName(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::NumericLiteral(_)
                    | TokenKind::Keyword(_)
                    | TokenKind::NullLiteral(_)
                    | TokenKind::PrivateIdentifier(_)
                    | TokenKind::Punctuator(
                        Punctuator::OpenBracket | Punctuator::Mul | Punctuator::OpenBlock,
                    ) => {
                        if contains_escape {
                            return Err(Error::general(
                                "keyword must not contain escaped characters",
                                token.span().start(),
                            ));
                        }
                        // this "static" is a keyword.
                        cursor.advance(interner);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        };

        let is_keyword = !matches!(
            cursor.peek(1, interner).or_abrupt()?.kind(),
            TokenKind::Punctuator(
                Punctuator::Assign
                    | Punctuator::CloseBlock
                    | Punctuator::OpenParen
                    | Punctuator::Semicolon
            )
        );

        let token = cursor.peek(0, interner).or_abrupt()?;
        let position = token.span().start();
        let element = match token.kind() {
            TokenKind::IdentifierName((Sym::CONSTRUCTOR, _)) if !r#static => {
                cursor.advance(interner);
                let strict = cursor.strict();
                cursor.set_strict(true);

                let parameters =
                    UniqueFormalParameters::new(false, false).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::OpenBlock),
                    "class constructor",
                    interner,
                )?;
                let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                cursor.expect(
                    TokenKind::Punctuator(Punctuator::CloseBlock),
                    "class constructor",
                    interner,
                )?;
                cursor.set_strict(strict);

                return Ok((
                    Some(FunctionExpression::new(self.name, parameters, body, false)),
                    None,
                ));
            }
            TokenKind::Punctuator(Punctuator::OpenBlock) if r#static => {
                cursor.advance(interner);
                let statement_list = if cursor
                    .next_if(TokenKind::Punctuator(Punctuator::CloseBlock), interner)?
                    .is_some()
                {
                    ast::StatementList::default()
                } else {
                    let strict = cursor.strict();
                    cursor.set_strict(true);
                    let position = cursor.peek(0, interner).or_abrupt()?.span().start();
                    let statement_list =
                        StatementList::new(false, true, false, &FUNCTION_BREAK_TOKENS, false, true)
                            .parse(cursor, interner)?;

                    let mut lexical_names = FxHashSet::default();

                    // It is a Syntax Error if the LexicallyDeclaredNames of
                    // ClassStaticBlockStatementList contains any duplicate entries.
                    for name in &lexically_declared_names(&statement_list) {
                        if !lexical_names.insert(*name) {
                            return Err(Error::general(
                                "lexical name declared multiple times",
                                position,
                            ));
                        }
                    }

                    // It is a Syntax Error if any element of the LexicallyDeclaredNames of
                    // ClassStaticBlockStatementList also occurs in the VarDeclaredNames of
                    // ClassStaticBlockStatementList.
                    for name in var_declared_names(&statement_list) {
                        if lexical_names.contains(&name) {
                            return Err(Error::general(
                                "lexical name declared in var names",
                                position,
                            ));
                        }
                    }

                    // It is a Syntax Error if ContainsDuplicateLabels of
                    // ClassStaticBlockStatementList with argument « » is true.
                    // It is a Syntax Error if ContainsUndefinedBreakTarget of
                    // ClassStaticBlockStatementList with argument « » is true.
                    // It is a Syntax Error if ContainsUndefinedContinueTarget of
                    // ClassStaticBlockStatementList with arguments « » and « » is true.
                    check_labels(&statement_list).map_err(|error| {
                        Error::lex(LexError::Syntax(error.message(interner).into(), position))
                    })?;

                    // It is a Syntax Error if ContainsArguments of ClassStaticBlockStatementList is true.
                    if contains_arguments(&statement_list) {
                        return Err(Error::general(
                            "'arguments' not allowed in class static block",
                            position,
                        ));
                    }

                    // It is a Syntax Error if ClassStaticBlockStatementList Contains SuperCall is true.
                    if contains(&statement_list, ContainsSymbol::SuperCall) {
                        return Err(Error::general("invalid super usage", position));
                    }

                    // It is a Syntax Error if ClassStaticBlockStatementList Contains await is true.
                    if contains(&statement_list, ContainsSymbol::AwaitExpression) {
                        return Err(Error::general("invalid await usage", position));
                    }

                    if contains_invalid_object_literal(&statement_list) {
                        return Err(Error::lex(LexError::Syntax(
                            "invalid object literal in class static block statement list".into(),
                            position,
                        )));
                    }

                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "class definition",
                        interner,
                    )?;
                    cursor.set_strict(strict);
                    statement_list
                };
                function::ClassElement::StaticBlock(function::FunctionBody::new(statement_list))
            }
            TokenKind::Punctuator(Punctuator::Mul) => {
                let token = cursor.peek(1, interner).or_abrupt()?;
                let name_position = token.span().start();
                if !r#static {
                    if let TokenKind::IdentifierName((Sym::CONSTRUCTOR, _)) = token.kind() {
                        return Err(Error::general(
                            "class constructor may not be a generator method",
                            token.span().start(),
                        ));
                    }
                }
                let strict = cursor.strict();
                cursor.set_strict(true);
                let (class_element_name, params, body) =
                    GeneratorMethod::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                cursor.set_strict(strict);

                let name = match class_element_name {
                    ClassElementName::PropertyName(name) => {
                        if r#static && name.literal() == Some(Sym::PROTOTYPE) {
                            return Err(Error::general(
                                "class may not have static method definitions named 'prototype'",
                                name_position,
                            ));
                        }
                        ClassElementName::PropertyName(name)
                    }
                    ClassElementName::PrivateName(name) => {
                        if name.description() == Sym::CONSTRUCTOR {
                            return Err(Error::general(
                                "class constructor may not be a private method",
                                name_position,
                            ));
                        }
                        ClassElementName::PrivateName(name)
                    }
                };
                function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                    name,
                    params,
                    body,
                    MethodDefinitionKind::Generator,
                    r#static,
                ))
            }
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
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Mul) => {
                        let token = cursor.peek(1, interner).or_abrupt()?;
                        let name_position = token.span().start();
                        match token.kind() {
                            TokenKind::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                                return Err(Error::general(
                                    "class constructor may not be a private method",
                                    token.span().start(),
                                ));
                            }
                            TokenKind::IdentifierName((Sym::CONSTRUCTOR, _)) if !r#static => {
                                return Err(Error::general(
                                    "class constructor may not be a generator method",
                                    token.span().start(),
                                ));
                            }
                            _ => {}
                        }
                        let strict = cursor.strict();
                        cursor.set_strict(true);
                        let (class_element_name, params, body) =
                            AsyncGeneratorMethod::new(self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?;
                        cursor.set_strict(strict);

                        let name = match class_element_name {
                            ClassElementName::PropertyName(name) => {
                                if r#static && name.literal() == Some(Sym::PROTOTYPE) {
                                    return Err(Error::general(
                                        "class may not have static method definitions named 'prototype'",
                                        name_position,
                                    ));
                                }
                                ClassElementName::PropertyName(name)
                            }
                            ClassElementName::PrivateName(name) => {
                                ClassElementName::PrivateName(name)
                            }
                        };

                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            name,
                            params,
                            body,
                            MethodDefinitionKind::AsyncGenerator,
                            r#static,
                        ))
                    }
                    TokenKind::IdentifierName((Sym::CONSTRUCTOR, _)) if !r#static => {
                        return Err(Error::general(
                            "class constructor may not be an async method",
                            token.span().start(),
                        ))
                    }
                    _ => {
                        let name_position = token.span().start();
                        let strict = cursor.strict();
                        cursor.set_strict(true);
                        let (class_element_name, params, body) =
                            AsyncMethod::new(self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?;
                        cursor.set_strict(strict);

                        let name = match class_element_name {
                            ClassElementName::PropertyName(name) => {
                                if r#static && name.literal() == Some(Sym::PROTOTYPE) {
                                    return Err(Error::general(
                                        "class may not have static method definitions named 'prototype'",
                                        name_position,
                                    ));
                                }
                                ClassElementName::PropertyName(name)
                            }
                            ClassElementName::PrivateName(name) => {
                                if r#static && name.description() == Sym::CONSTRUCTOR {
                                    return Err(Error::general(
                                        "class constructor may not be a private method",
                                        name_position,
                                    ));
                                }
                                ClassElementName::PrivateName(name)
                            }
                        };
                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            name,
                            params,
                            body,
                            MethodDefinitionKind::Async,
                            r#static,
                        ))
                    }
                }
            }
            TokenKind::IdentifierName((Sym::GET | Sym::SET, ContainsEscapeSequence(true)))
                if is_keyword =>
            {
                return Err(Error::general(
                    "keyword must not contain escaped characters",
                    token.span().start(),
                ))
            }
            TokenKind::IdentifierName((Sym::GET, ContainsEscapeSequence(false))) if is_keyword => {
                cursor.advance(interner);
                let token = cursor.peek(0, interner).or_abrupt()?;
                match token.kind() {
                    TokenKind::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                        return Err(Error::general(
                            "class constructor may not be a private method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::PrivateIdentifier(name) => {
                        let name = *name;
                        cursor.advance(interner);
                        let strict = cursor.strict();
                        cursor.set_strict(true);
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
                            return Err(Error::lex(LexError::Syntax(
                            "Illegal 'use strict' directive in function with non-simple parameter list"
                                .into(),
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict(strict);
                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            ClassElementName::PrivateName(PrivateName::new(name)),
                            params,
                            body,
                            MethodDefinitionKind::Get,
                            r#static,
                        ))
                    }
                    TokenKind::IdentifierName((Sym::CONSTRUCTOR, _)) if !r#static => {
                        return Err(Error::general(
                            "class constructor may not be a getter method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::IdentifierName(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::NumericLiteral(_)
                    | TokenKind::Keyword(_)
                    | TokenKind::NullLiteral(_)
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
                        let strict = cursor.strict();
                        cursor.set_strict(true);
                        let body = FunctionBody::new(false, false).parse(cursor, interner)?;
                        cursor.set_strict(strict);
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::CloseBlock),
                            "class getter",
                            interner,
                        )?;
                        if r#static && name.literal() == Some(Sym::PROTOTYPE) {
                            return Err(Error::general(
                                "class may not have static method definitions named 'prototype'",
                                name_position,
                            ));
                        }
                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            ClassElementName::PropertyName(name),
                            FormalParameterList::default(),
                            body,
                            MethodDefinitionKind::Get,
                            r#static,
                        ))
                    }
                    _ => {
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            function::ClassElement::StaticFieldDefinition(
                                ast::property::PropertyName::Literal(Sym::GET),
                                None,
                            )
                        } else {
                            function::ClassElement::FieldDefinition(
                                ast::property::PropertyName::Literal(Sym::GET),
                                None,
                            )
                        }
                    }
                }
            }
            TokenKind::IdentifierName((Sym::SET, ContainsEscapeSequence(false))) if is_keyword => {
                cursor.advance(interner);
                let token = cursor.peek(0, interner).or_abrupt()?;
                match token.kind() {
                    TokenKind::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                        return Err(Error::general(
                            "class constructor may not be a private method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::PrivateIdentifier(name) => {
                        let name = *name;
                        cursor.advance(interner);
                        let strict = cursor.strict();
                        cursor.set_strict(true);
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
                            return Err(Error::lex(LexError::Syntax(
                            "Illegal 'use strict' directive in function with non-simple parameter list"
                                .into(),
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict(strict);
                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            ClassElementName::PrivateName(PrivateName::new(name)),
                            params,
                            body,
                            MethodDefinitionKind::Set,
                            r#static,
                        ))
                    }
                    TokenKind::IdentifierName((Sym::CONSTRUCTOR, _)) if !r#static => {
                        return Err(Error::general(
                            "class constructor may not be a setter method",
                            token.span().start(),
                        ))
                    }
                    TokenKind::IdentifierName(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::NumericLiteral(_)
                    | TokenKind::Keyword(_)
                    | TokenKind::NullLiteral(_)
                    | TokenKind::Punctuator(Punctuator::OpenBracket) => {
                        let name_position = token.span().start();
                        let name = PropertyName::new(self.allow_yield, self.allow_await)
                            .parse(cursor, interner)?;
                        let strict = cursor.strict();
                        cursor.set_strict(true);
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
                            return Err(Error::lex(LexError::Syntax(
                            "Illegal 'use strict' directive in function with non-simple parameter list"
                                .into(),
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict(strict);
                        if r#static && name.literal() == Some(Sym::PROTOTYPE) {
                            return Err(Error::general(
                                "class may not have static method definitions named 'prototype'",
                                name_position,
                            ));
                        }
                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            ClassElementName::PropertyName(name),
                            params,
                            body,
                            MethodDefinitionKind::Set,
                            r#static,
                        ))
                    }
                    _ => {
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            function::ClassElement::StaticFieldDefinition(
                                ast::property::PropertyName::Literal(Sym::SET),
                                None,
                            )
                        } else {
                            function::ClassElement::FieldDefinition(
                                ast::property::PropertyName::Literal(Sym::SET),
                                None,
                            )
                        }
                    }
                }
            }
            TokenKind::PrivateIdentifier(Sym::CONSTRUCTOR) => {
                return Err(Error::general(
                    "class constructor may not be a private method",
                    token.span().start(),
                ))
            }
            TokenKind::PrivateIdentifier(name) => {
                let name = *name;
                cursor.advance(interner);
                let token = cursor.peek(0, interner).or_abrupt()?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Assign) => {
                        cursor.advance(interner);
                        let strict = cursor.strict();
                        cursor.set_strict(true);
                        let mut rhs =
                            AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?;
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        cursor.set_strict(strict);
                        let function_name = interner.get_or_intern(
                            [utf16!("#"), interner.resolve_expect(name).utf16()]
                                .concat()
                                .as_slice(),
                        );
                        rhs.set_anonymous_function_definition_name(&Identifier::new(function_name));
                        if r#static {
                            function::ClassElement::PrivateStaticFieldDefinition(
                                PrivateName::new(name),
                                Some(rhs),
                            )
                        } else {
                            function::ClassElement::PrivateFieldDefinition(
                                PrivateName::new(name),
                                Some(rhs),
                            )
                        }
                    }
                    TokenKind::Punctuator(Punctuator::OpenParen) => {
                        let strict = cursor.strict();
                        cursor.set_strict(true);
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
                            return Err(Error::lex(LexError::Syntax(
                                "Illegal 'use strict' directive in function with non-simple parameter list".into(),
                                token.span().start(),
                            )));
                        }
                        cursor.set_strict(strict);
                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            ClassElementName::PrivateName(PrivateName::new(name)),
                            params,
                            body,
                            MethodDefinitionKind::Ordinary,
                            r#static,
                        ))
                    }
                    _ => {
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            function::ClassElement::PrivateStaticFieldDefinition(
                                PrivateName::new(name),
                                None,
                            )
                        } else {
                            function::ClassElement::PrivateFieldDefinition(
                                PrivateName::new(name),
                                None,
                            )
                        }
                    }
                }
            }
            TokenKind::IdentifierName(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::NumericLiteral(_)
            | TokenKind::Keyword(_)
            | TokenKind::NullLiteral(_)
            | TokenKind::Punctuator(Punctuator::OpenBracket) => {
                let name_position = token.span().start();
                let name = PropertyName::new(self.allow_yield, self.allow_await)
                    .parse(cursor, interner)?;
                let token = cursor.peek(0, interner).or_abrupt()?;
                match token.kind() {
                    TokenKind::Punctuator(Punctuator::Assign) => {
                        if let Some(name) = name.literal() {
                            if r#static {
                                if [Sym::CONSTRUCTOR, Sym::PROTOTYPE].contains(&name) {
                                    return Err(Error::general(
                                        "class may not have static field definitions named 'constructor' or 'prototype'",
                                        name_position,
                                    ));
                                }
                            } else if name == Sym::CONSTRUCTOR {
                                return Err(Error::general(
                                    "class may not have field definitions named 'constructor'",
                                    name_position,
                                ));
                            }
                        }
                        cursor.advance(interner);
                        let strict = cursor.strict();
                        cursor.set_strict(true);
                        let mut rhs =
                            AssignmentExpression::new(true, self.allow_yield, self.allow_await)
                                .parse(cursor, interner)?;
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        cursor.set_strict(strict);
                        if let Some(name) = name.literal() {
                            rhs.set_anonymous_function_definition_name(&Identifier::new(name));
                        }
                        if r#static {
                            function::ClassElement::StaticFieldDefinition(name, Some(rhs))
                        } else {
                            function::ClassElement::FieldDefinition(name, Some(rhs))
                        }
                    }
                    TokenKind::Punctuator(Punctuator::OpenParen) => {
                        if r#static && name.literal() == Some(Sym::PROTOTYPE) {
                            return Err(Error::general(
                                "class may not have static method definitions named 'prototype'",
                                name_position,
                            ));
                        }
                        let strict = cursor.strict();
                        cursor.set_strict(true);
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
                            return Err(Error::lex(LexError::Syntax(
                            "Illegal 'use strict' directive in function with non-simple parameter list"
                                .into(),
                                token.span().start(),
                        )));
                        }
                        cursor.set_strict(strict);
                        function::ClassElement::MethodDefinition(ClassMethodDefinition::new(
                            ClassElementName::PropertyName(name),
                            params,
                            body,
                            MethodDefinitionKind::Ordinary,
                            r#static,
                        ))
                    }
                    _ => {
                        if let Some(name) = name.literal() {
                            if r#static {
                                if [Sym::CONSTRUCTOR, Sym::PROTOTYPE].contains(&name) {
                                    return Err(Error::general(
                                        "class may not have static field definitions named 'constructor' or 'prototype'",
                                        name_position,
                                    ));
                                }
                            } else if name == Sym::CONSTRUCTOR {
                                return Err(Error::general(
                                    "class may not have field definitions named 'constructor'",
                                    name_position,
                                ));
                            }
                        }
                        cursor.expect_semicolon("expected semicolon", interner)?;
                        if r#static {
                            function::ClassElement::StaticFieldDefinition(name, None)
                        } else {
                            function::ClassElement::FieldDefinition(name, None)
                        }
                    }
                }
            }
            _ => return Err(Error::general("unexpected token", token.span().start())),
        };

        match &element {
            // FieldDefinition : ClassElementName Initializer [opt]
            // It is a Syntax Error if Initializer is present and ContainsArguments of Initializer is true.
            function::ClassElement::FieldDefinition(_, Some(node))
            | function::ClassElement::StaticFieldDefinition(_, Some(node))
            | function::ClassElement::PrivateFieldDefinition(_, Some(node))
            | function::ClassElement::PrivateStaticFieldDefinition(_, Some(node)) => {
                if contains_arguments(node) {
                    return Err(Error::general(
                        "'arguments' not allowed in class field definition",
                        position,
                    ));
                }
            }

            _ => {}
        }

        Ok((None, Some(element)))
    }
}
