//! Function definition parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
//! [spec]: https://tc39.es/ecma262/#sec-function-definitions

#[cfg(test)]
mod tests;

use crate::{
    gc::{Finalize, Trace},
    syntax::{
        ast::{
            node::{FunctionDecl, Node},
            Keyword, Punctuator,
        },
        lexer::{InputElement, TokenKind},
        parser::{
            expression::Expression,
            function::{FormalParameters, FunctionBody},
            statement::BindingIdentifier,
            AllowAwait, AllowYield, Cursor, ParseError, TokenParser,
        },
    },
    BoaProfiler,
};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum ClassField {
    /// A method on a class.
    Method(FunctionDecl),
    /// A field on a class (includes an initializer)
    // TODO: Name should be a VariableDeclList (I think)
    Field(Box<str>, Node),
    /// A getter function. This will never take any arguments.
    Getter(FunctionDecl),
    /// A setter function. This will always take an argument.
    Setter(FunctionDecl),
}

/// Formal class element list parsing.
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/class
/// [spec]: https://tc39.es/ecma262/#prod-ClassElementList
#[derive(Debug, Clone, Copy)]
pub(in crate::syntax::parser) struct ClassElementList {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
}

impl ClassElementList {
    /// Creates a new `FormalElements` parser.
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

impl<R> TokenParser<R> for ClassElementList
where
    R: Read,
{
    type Output = (
        Option<FunctionDecl>, // Constructor
        Box<[ClassField]>,    // Methods/fields
        Box<[ClassField]>,    // Static methods/fields
    );

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Self::Output, ParseError> {
        let _timer = BoaProfiler::global().start_event("ClassElementList", "Parsing");
        cursor.set_goal(InputElement::RegExp);

        // Field and method names. Used to check for duplicate functions/fields.
        let mut field_names = HashSet::new();
        let mut getter_names = HashMap::new();
        let mut setter_names = HashMap::new();

        let mut constructor = None;
        let mut fields = Vec::new();
        let mut static_fields = Vec::new();

        if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
            == &TokenKind::Punctuator(Punctuator::CloseBlock)
        {
            return Ok((
                None,
                fields.into_boxed_slice(),
                static_fields.into_boxed_slice(),
            ));
        }

        loop {
            let next = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
            let is_static = match next.kind() {
                TokenKind::Keyword(Keyword::Static) => {
                    // Consume the static token.
                    cursor.next()?;
                    true
                }
                _ => false,
            };

            let is_getter;
            let is_setter;

            // No matter if there was a static token, a `get` or `set` token is valid.
            let next = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;
            match next.kind() {
                TokenKind::Keyword(Keyword::Get) => {
                    // Consume the get token.
                    cursor.next()?;
                    is_getter = true;
                    is_setter = false;
                }
                TokenKind::Keyword(Keyword::Set) => {
                    // Consume the set token.
                    cursor.next()?;
                    is_getter = false;
                    is_setter = true;
                }
                _ => {
                    is_getter = false;
                    is_setter = false;
                }
            };

            // TODO: Parse async/yeild here

            // TODO: This should sometimes be parsed as a let decl list
            let position = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
            let name = BindingIdentifier::new(self.allow_yield, self.allow_await).parse(cursor)?;
            if *name == *"constructor" {
                if constructor.is_some() {
                    return Err(ParseError::general(
                        "Cannot have multiple constructors on an object",
                        position,
                    ));
                }
            } else if *name == *"prototype" && is_static {
                return Err(ParseError::general(
                    "The name `prototype` is reserved for static fields",
                    position,
                ));
            }

            let next = cursor.next()?.ok_or(ParseError::AbruptEnd)?;
            let name_pos = next.span().start();
            let field = match next.kind() {
                // A method definition
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let arg_pos = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.span().start();
                    let params = FormalParameters::new(false, false).parse(cursor)?;

                    let mut names = HashSet::new();
                    for p in params.iter() {
                        if !names.insert(p.name()) {
                            return Err(ParseError::general("Duplicate argument name", position));
                        }
                    }

                    // This is only partially correct. A method can enable strict mode with "using strict"; which is not handled here.
                    if let Some(last) = params.last() {
                        if cursor.strict_mode() && last.is_rest_param() {
                            return Err(ParseError::general(
                                "Cannot have spread parameters on a class method in strict mode",
                                position,
                            ));
                        }
                    }

                    cursor.expect(Punctuator::CloseParen, "class function declaration")?;
                    cursor.expect(Punctuator::OpenBlock, "class function declaration")?;

                    let body =
                        FunctionBody::new(self.allow_yield, self.allow_await).parse(cursor)?;

                    cursor.expect(Punctuator::CloseBlock, "class function declaration")?;

                    if *name == *"constructor" {
                        if is_getter {
                            return Err(ParseError::general(
                                "Cannot create a getter named `constructor`",
                                name_pos,
                            ));
                        } else if is_setter {
                            return Err(ParseError::general(
                                "Cannot create a setter named `constructor`",
                                name_pos,
                            ));
                        } else if is_static {
                            return Err(ParseError::general(
                                "Cannot create a static function named `constructor`",
                                name_pos,
                            ));
                        }
                        constructor = Some(FunctionDecl::new(name, params, body));
                        None
                    } else if is_getter {
                        // This is a getter, so a setter with the same name is valid.
                        if field_names.contains(&name) || getter_names.contains_key(&name) {
                            return Err(ParseError::general("Duplicate getter name", name_pos));
                        }
                        if setter_names.get(&name) == Some(&!is_static) {
                            if is_static {
                                return Err(ParseError::general(
                                    "A static setter cannot have a non-static getter",
                                    name_pos,
                                ));
                            } else {
                                return Err(ParseError::general(
                                    "A non-static setter cannot have a static getter",
                                    name_pos,
                                ));
                            }
                        }
                        if !params.is_empty() {
                            return Err(ParseError::general("Getters take no arguments", arg_pos));
                        }
                        getter_names.insert(name.clone(), is_static);
                        Some(ClassField::Getter(FunctionDecl::new(name, params, body)))
                    } else if is_setter {
                        // This is a setter, so a getter with the same name is valid.
                        if field_names.contains(&name) || setter_names.contains_key(&name) {
                            return Err(ParseError::general("Duplicate setter name", name_pos));
                        }
                        if getter_names.get(&name) == Some(&!is_static) {
                            if is_static {
                                return Err(ParseError::general(
                                    "A static getter cannot have a non-static setter",
                                    name_pos,
                                ));
                            } else {
                                return Err(ParseError::general(
                                    "A non-static getter cannot have a static setter",
                                    name_pos,
                                ));
                            }
                        }
                        setter_names.insert(name.clone(), is_static);
                        Some(ClassField::Setter(FunctionDecl::new(name, params, body)))
                    } else {
                        if field_names.contains(&name)
                            || getter_names.contains_key(&name)
                            || setter_names.contains_key(&name)
                        {
                            return Err(ParseError::general("Duplicate method name", name_pos));
                        }
                        field_names.insert(name.clone());
                        Some(ClassField::Method(FunctionDecl::new(name, params, body)))
                    }
                }
                // A field definition
                TokenKind::Punctuator(Punctuator::Assign) => {
                    if *name == *"constructor" {
                        return Err(ParseError::general(
                            "Fields cannot be named `constructor`",
                            name_pos,
                        ));
                    }
                    // Field definitions cannot proceed a `get` or `set`
                    if is_getter {
                        return Err(ParseError::unexpected(next, "after `get`"));
                    } else if is_setter {
                        return Err(ParseError::unexpected(next, "after `set`"));
                    }
                    let value =
                        Expression::new(true, self.allow_yield, self.allow_await).parse(cursor)?;
                    // Classes are always parsed in strict mode, so this is always a requirement.
                    cursor.expect_semicolon("after a class field declaration")?;

                    if field_names.contains(&name)
                        || getter_names.contains_key(&name)
                        || setter_names.contains_key(&name)
                    {
                        return Err(ParseError::general("Duplicate field name", name_pos));
                    }
                    field_names.insert(name.clone());
                    Some(ClassField::Field(name, value))
                }
                _ => {
                    return Err(ParseError::expected(
                        vec![
                            TokenKind::Punctuator(Punctuator::OpenParen),
                            TokenKind::Punctuator(Punctuator::Assign),
                        ],
                        next,
                        "class method or field declatation",
                    ))
                }
            };

            if let Some(f) = field {
                if is_static {
                    static_fields.push(f);
                } else {
                    fields.push(f);
                }
            }

            if cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?.kind()
                == &TokenKind::Punctuator(Punctuator::CloseBlock)
            {
                break;
            }
        }

        Ok((
            constructor,
            fields.into_boxed_slice(),
            static_fields.into_boxed_slice(),
        ))
    }
}
