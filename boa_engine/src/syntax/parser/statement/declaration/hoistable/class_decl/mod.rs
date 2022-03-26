use crate::syntax::{
    ast::{
        node::{
            self,
            declaration::class_decl::ClassElement,
            object::{MethodDefinition, PropertyName::Literal},
            Class, FormalParameterList, FunctionExpr,
        },
        Keyword, Punctuator,
    },
    lexer::{Error as LexError, TokenKind},
    parser::{
        expression::{
            AssignmentExpression, AsyncGeneratorMethod, AsyncMethod, GeneratorMethod,
            LeftHandSideExpression, PropertyName,
        },
        function::{
            FormalParameter, FormalParameters, FunctionBody, UniqueFormalParameters,
            FUNCTION_BREAK_TOKENS,
        },
        statement::{BindingIdentifier, StatementList},
        AllowAwait, AllowDefault, AllowYield, Cursor, ParseError, TokenParser,
    },
};
use boa_interner::{Interner, Sym};
use node::Node;
use rustc_hash::FxHashMap;
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
        cursor.expect(Keyword::Class, "class declaration", interner)?;
        let strict = cursor.strict_mode();
        cursor.set_strict_mode(true);

        let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
        let name = match token.kind() {
            TokenKind::Identifier(_) | TokenKind::Keyword(Keyword::Yield | Keyword::Await) => {
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
        cursor.push_private_environment();

        let super_ref = if cursor
            .next_if(TokenKind::keyword(Keyword::Extends), interner)?
            .is_some()
        {
            let strict = cursor.strict_mode();
            cursor.set_strict_mode(true);
            let lhs = LeftHandSideExpression::new(None, self.allow_yield, self.allow_await)
                .parse(cursor, interner)?;
            cursor.set_strict_mode(strict);
            Some(Box::new(lhs))
        } else {
            None
        };

        cursor.expect(Punctuator::OpenBlock, "class tail", interner)?;

        let mut constructor = None;
        let mut elements = Vec::new();
        let mut r#static = false;
        let mut private_elements_names = FxHashMap::default();

        // The identifier "static" is forbidden in strict mode but used as a keyword in classes.
        // Because of this, strict mode has to temporarily be disabled while parsing class field names.
        let strict = cursor.strict_mode();
        cursor.set_strict_mode(false);
        loop {
            let token = cursor.peek(0, interner)?.ok_or(ParseError::AbruptEnd)?;
            match token.kind() {
                TokenKind::Punctuator(Punctuator::CloseBlock) if !r#static => {
                    cursor.next(interner).expect("token disappeared");
                    break;
                }
                TokenKind::Punctuator(Punctuator::Semicolon) => {
                    cursor.next(interner).expect("token disappeared");
                    if r#static {
                        elements.push(ClassElement::FieldDefinition(Sym::STATIC.into(), None));
                    }
                }
                TokenKind::Identifier(Sym::STATIC) if !r#static => {
                    cursor.next(interner).expect("token disappeared");
                    let token = cursor
                        .peek(0, interner)?
                        .ok_or(ParseError::AbruptEnd)?
                        .clone();
                    let strict = cursor.strict_mode();
                    cursor.set_strict_mode(true);
                    match token.kind() {
                        TokenKind::Punctuator(Punctuator::Assign) => {
                            cursor.next(interner).expect("token disappeared");
                            let rhs = AssignmentExpression::new(
                                Sym::STATIC,
                                true,
                                self.allow_yield,
                                self.allow_await,
                            )
                            .parse(cursor, interner)?;
                            cursor.expect_semicolon("expected semicolon", interner)?;
                            elements.push(ClassElement::FieldDefinition(
                                Literal(Sym::STATIC),
                                Some(rhs),
                            ));
                        }
                        TokenKind::Punctuator(Punctuator::OpenParen) => {
                            let params = UniqueFormalParameters::new(false, false)
                                .parse(cursor, interner)?;
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

                            elements
                                .push(ClassElement::MethodDefinition(Literal(Sym::STATIC), method));
                        }
                        _ => {
                            r#static = true;
                        }
                    }
                    cursor.set_strict_mode(strict);
                    continue;
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
                        let statement_list =
                            StatementList::new(false, true, false, true, &FUNCTION_BREAK_TOKENS)
                                .parse(cursor, interner)?;
                        cursor.expect(
                            TokenKind::Punctuator(Punctuator::CloseBlock),
                            "class definition",
                            interner,
                        )?;
                        cursor.set_strict_mode(strict);
                        statement_list
                    };
                    elements.push(ClassElement::StaticBlock(statement_list));
                }
                TokenKind::Identifier(Sym::CONSTRUCTOR) => {
                    if constructor.is_some() {
                        return Err(ParseError::general(
                            "a class may only have one constructor",
                            token.span().start(),
                        ));
                    }
                    let strict = cursor.strict_mode();
                    cursor.set_strict_mode(true);
                    cursor.next(interner).expect("token disappeared");
                    cursor.expect(Punctuator::OpenParen, "class definition", interner)?;
                    let parameters = FormalParameters::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(Punctuator::CloseParen, "class definition", interner)?;
                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::OpenBlock),
                        "class definition",
                        interner,
                    )?;
                    let body = FunctionBody::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(
                        TokenKind::Punctuator(Punctuator::CloseBlock),
                        "class definition",
                        interner,
                    )?;
                    constructor = Some(FunctionExpr::new(self.name, parameters, body));
                    cursor.set_strict_mode(strict);
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
                    let (property_name, method) =
                        GeneratorMethod::new(self.allow_yield, self.allow_await)
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
                        elements.push(ClassElement::StaticMethodDefinition(property_name, method));
                    } else {
                        elements.push(ClassElement::MethodDefinition(property_name, method));
                    }
                }
                TokenKind::Keyword(Keyword::Async) => {
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
                                elements.push(ClassElement::StaticMethodDefinition(
                                    property_name,
                                    method,
                                ));
                            } else {
                                elements
                                    .push(ClassElement::MethodDefinition(property_name, method));
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
                                elements.push(ClassElement::StaticMethodDefinition(
                                    property_name,
                                    method,
                                ));
                            } else {
                                elements
                                    .push(ClassElement::MethodDefinition(property_name, method));
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
                                elements.push(ClassElement::StaticFieldDefinition(
                                    Literal(Sym::GET),
                                    Some(rhs),
                                ));
                            } else {
                                elements.push(ClassElement::FieldDefinition(
                                    Literal(Sym::GET),
                                    Some(rhs),
                                ));
                            }
                        }
                        TokenKind::Punctuator(Punctuator::OpenParen) => {
                            let strict = cursor.strict_mode();
                            cursor.set_strict_mode(true);
                            let params = UniqueFormalParameters::new(false, false)
                                .parse(cursor, interner)?;
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
                                elements.push(ClassElement::StaticMethodDefinition(
                                    Literal(Sym::GET),
                                    method,
                                ));
                            } else {
                                elements.push(ClassElement::MethodDefinition(
                                    Literal(Sym::GET),
                                    method,
                                ));
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
                            let start = token.span().start();
                            cursor.next(interner).expect("token disappeared");
                            let strict = cursor.strict_mode();
                            cursor.set_strict_mode(true);
                            let params = UniqueFormalParameters::new(false, false)
                                .parse(cursor, interner)?;
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
                                MethodDefinition::Get(FunctionExpr::new(None, params, body));
                            if r#static {
                                match private_elements_names.get(&name) {
                                    Some(PrivateElement::StaticSetter) => {
                                        private_elements_names
                                            .insert(name, PrivateElement::StaticValue);
                                    }
                                    Some(_) => {
                                        return Err(ParseError::general(
                                            "private identifier has already been declared",
                                            start,
                                        ));
                                    }
                                    None => {
                                        private_elements_names
                                            .insert(name, PrivateElement::StaticGetter);
                                    }
                                }
                                elements.push(ClassElement::PrivateStaticMethodDefinition(
                                    name, method,
                                ));
                            } else {
                                match private_elements_names.get(&name) {
                                    Some(PrivateElement::Setter) => {
                                        private_elements_names.insert(name, PrivateElement::Value);
                                    }
                                    Some(_) => {
                                        return Err(ParseError::general(
                                            "private identifier has already been declared",
                                            start,
                                        ));
                                    }
                                    None => {
                                        private_elements_names.insert(name, PrivateElement::Getter);
                                    }
                                }
                                elements.push(ClassElement::PrivateMethodDefinition(name, method));
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
                                elements.push(ClassElement::StaticMethodDefinition(name, method));
                            } else {
                                elements.push(ClassElement::MethodDefinition(name, method));
                            }
                        }
                        _ => {
                            cursor.expect_semicolon("expected semicolon", interner)?;
                            if r#static {
                                elements.push(ClassElement::StaticFieldDefinition(
                                    Literal(Sym::GET),
                                    None,
                                ));
                            } else {
                                elements
                                    .push(ClassElement::FieldDefinition(Literal(Sym::GET), None));
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
                                elements.push(ClassElement::StaticFieldDefinition(
                                    Literal(Sym::SET),
                                    Some(rhs),
                                ));
                            } else {
                                elements.push(ClassElement::FieldDefinition(
                                    Literal(Sym::SET),
                                    Some(rhs),
                                ));
                            }
                        }
                        TokenKind::Punctuator(Punctuator::OpenParen) => {
                            cursor.next(interner).expect("token disappeared");
                            let strict = cursor.strict_mode();
                            cursor.set_strict_mode(true);
                            let parameters: FormalParameterList =
                                FormalParameter::new(false, false)
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
                            let method = MethodDefinition::Ordinary(FunctionExpr::new(
                                None, parameters, body,
                            ));
                            if r#static {
                                elements.push(ClassElement::StaticMethodDefinition(
                                    Literal(Sym::SET),
                                    method,
                                ));
                            } else {
                                elements.push(ClassElement::MethodDefinition(
                                    Literal(Sym::SET),
                                    method,
                                ));
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
                            let start = token.span().start();
                            cursor.next(interner).expect("token disappeared");
                            let strict = cursor.strict_mode();
                            cursor.set_strict_mode(true);
                            let params = UniqueFormalParameters::new(false, false)
                                .parse(cursor, interner)?;
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
                                MethodDefinition::Set(FunctionExpr::new(None, params, body));
                            if r#static {
                                match private_elements_names.get(&name) {
                                    Some(PrivateElement::StaticGetter) => {
                                        private_elements_names
                                            .insert(name, PrivateElement::StaticValue);
                                    }
                                    Some(_) => {
                                        return Err(ParseError::general(
                                            "private identifier has already been declared",
                                            start,
                                        ));
                                    }
                                    None => {
                                        private_elements_names
                                            .insert(name, PrivateElement::StaticSetter);
                                    }
                                }
                                elements.push(ClassElement::PrivateStaticMethodDefinition(
                                    name, method,
                                ));
                            } else {
                                match private_elements_names.get(&name) {
                                    Some(PrivateElement::Getter) => {
                                        private_elements_names.insert(name, PrivateElement::Value);
                                    }
                                    Some(_) => {
                                        return Err(ParseError::general(
                                            "private identifier has already been declared",
                                            start,
                                        ));
                                    }
                                    None => {
                                        private_elements_names.insert(name, PrivateElement::Setter);
                                    }
                                }
                                elements.push(ClassElement::PrivateMethodDefinition(name, method));
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
                            let params = UniqueFormalParameters::new(false, false)
                                .parse(cursor, interner)?;
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
                                MethodDefinition::Set(FunctionExpr::new(None, params, body));
                            if r#static {
                                if let Some(name) = name.prop_name() {
                                    if name == Sym::PROTOTYPE {
                                        return Err(ParseError::general(
                                                "class may not have static method definitions named 'prototype'",
                                                name_position,
                                            ));
                                    }
                                }
                                elements.push(ClassElement::StaticMethodDefinition(name, method));
                            } else {
                                elements.push(ClassElement::MethodDefinition(name, method));
                            }
                        }
                        _ => {
                            cursor.expect_semicolon("expected semicolon", interner)?;
                            if r#static {
                                elements.push(ClassElement::StaticFieldDefinition(
                                    Literal(Sym::SET),
                                    None,
                                ));
                            } else {
                                elements
                                    .push(ClassElement::FieldDefinition(Literal(Sym::SET), None));
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
                    let start = token.span().start();
                    cursor.next(interner).expect("token disappeared");
                    if private_elements_names.contains_key(&name) {
                        return Err(ParseError::general(
                            "private identifier has already been declared",
                            start,
                        ));
                    }
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
                            if r#static {
                                private_elements_names.insert(name, PrivateElement::StaticValue);
                                elements.push(ClassElement::PrivateStaticFieldDefinition(
                                    name,
                                    Some(rhs),
                                ));
                            } else {
                                private_elements_names.insert(name, PrivateElement::Value);
                                elements
                                    .push(ClassElement::PrivateFieldDefinition(name, Some(rhs)));
                            }
                            cursor.set_strict_mode(strict);
                        }
                        TokenKind::Punctuator(Punctuator::OpenParen) => {
                            let strict = cursor.strict_mode();
                            cursor.set_strict_mode(true);
                            let params = UniqueFormalParameters::new(false, false)
                                .parse(cursor, interner)?;
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

                            private_elements_names.insert(name, PrivateElement::Method);
                            if r#static {
                                elements.push(ClassElement::PrivateStaticMethodDefinition(
                                    name, method,
                                ));
                            } else {
                                elements.push(ClassElement::PrivateMethodDefinition(name, method));
                            }
                            cursor.set_strict_mode(strict);
                        }
                        _ => {
                            cursor.expect_semicolon("expected semicolon", interner)?;
                            if r#static {
                                private_elements_names.insert(name, PrivateElement::StaticValue);
                                elements
                                    .push(ClassElement::PrivateStaticFieldDefinition(name, None));
                            } else {
                                private_elements_names.insert(name, PrivateElement::Value);
                                elements.push(ClassElement::PrivateFieldDefinition(name, None));
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
                            if r#static {
                                elements.push(ClassElement::StaticFieldDefinition(name, Some(rhs)));
                            } else {
                                elements.push(ClassElement::FieldDefinition(name, Some(rhs)));
                            }
                            cursor.set_strict_mode(strict);
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
                            let params = UniqueFormalParameters::new(false, false)
                                .parse(cursor, interner)?;
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
                            if r#static {
                                elements.push(ClassElement::StaticMethodDefinition(name, method));
                            } else {
                                elements.push(ClassElement::MethodDefinition(name, method));
                            }
                            cursor.set_strict_mode(strict);
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
                                elements.push(ClassElement::StaticFieldDefinition(name, None));
                            } else {
                                elements.push(ClassElement::FieldDefinition(name, None));
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
            }
            r#static = false;
        }

        cursor.set_strict_mode(strict);
        cursor.pop_private_environment(&private_elements_names)?;

        Ok(Class::new(self.name, super_ref, constructor, elements))
    }
}

/// Representation of private object elements.
#[derive(Debug, PartialEq)]
pub(in crate::syntax) enum PrivateElement {
    Method,
    Value,
    Getter,
    Setter,
    StaticValue,
    StaticSetter,
    StaticGetter,
}
