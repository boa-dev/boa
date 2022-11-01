use std::borrow::Cow;
use std::ops::ControlFlow;

use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    string::ToStringEscaped,
    syntax::ast::{
        block_to_string,
        expression::{Expression, Identifier},
        join_nodes,
        property::{MethodDefinition, PropertyName},
        ContainsSymbol, Declaration, StatementList, StatementListItem,
    },
    try_break,
};
use boa_interner::{Interner, Sym, ToIndentedString, ToInternedString};

use super::Function;

/// A class declaration, as defined by the [spec].
///
/// A [class][mdn] declaration defines a class with the specified methods, fields, and optional constructor.
/// Classes can be used to create objects, which can also be created through literals (using `{}`).
///
/// [spec]: https://tc39.es/ecma262/#sec-class-definitions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    name: Option<Identifier>,
    super_ref: Option<Expression>,
    constructor: Option<Function>,
    elements: Box<[ClassElement]>,
}

impl Class {
    /// Creates a new class declaration.
    #[inline]
    pub(in crate::syntax) fn new(
        name: Option<Identifier>,
        super_ref: Option<Expression>,
        constructor: Option<Function>,
        elements: Box<[ClassElement]>,
    ) -> Self {
        Self {
            name,
            super_ref,
            constructor,
            elements,
        }
    }

    /// Returns the name of the class.
    #[inline]
    pub(crate) fn name(&self) -> Option<Identifier> {
        self.name
    }

    /// Returns the super class ref of the class.
    #[inline]
    pub(crate) fn super_ref(&self) -> Option<&Expression> {
        self.super_ref.as_ref()
    }

    /// Returns the constructor of the class.
    #[inline]
    pub(crate) fn constructor(&self) -> Option<&Function> {
        self.constructor.as_ref()
    }

    /// Gets the list of all fields defined on the class.
    #[inline]
    pub(crate) fn elements(&self) -> &[ClassElement] {
        &self.elements
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        matches!(self.name, Some(ref ident) if *ident == Sym::ARGUMENTS)
            || matches!(self.super_ref, Some(ref expr) if expr.contains_arguments())
            || self.elements.iter().any(ClassElement::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        if symbol == ContainsSymbol::ClassBody && !self.elements.is_empty() {
            return true;
        }
        if symbol == ContainsSymbol::ClassHeritage {
            return self.super_ref.is_some();
        }

        matches!(self.super_ref, Some(ref expr) if expr.contains(symbol))
            || self.elements.iter().any(|elem| elem.contains(symbol))
    }
}

impl ToIndentedString for Class {
    fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let class_name = self.name.map_or(Cow::Borrowed(""), |s| {
            interner.resolve_expect(s.sym()).join(
                Cow::Borrowed,
                |utf16| Cow::Owned(utf16.to_string_escaped()),
                true,
            )
        });
        if self.elements.is_empty() && self.constructor().is_none() {
            return format!(
                "class {class_name}{} {{}}",
                if let Some(sup) = &self.super_ref {
                    format!(" extends {}", sup.to_interned_string(interner))
                } else {
                    "".to_string()
                }
            );
        }
        let indentation = "    ".repeat(indent_n + 1);
        let mut buf = format!(
            "class {class_name}{} {{\n",
            if let Some(sup) = &self.super_ref {
                format!("extends {}", sup.to_interned_string(interner))
            } else {
                "".to_string()
            }
        );
        if let Some(expr) = &self.constructor {
            buf.push_str(&format!(
                "{indentation}constructor({}) {}\n",
                join_nodes(interner, &expr.parameters().parameters),
                block_to_string(expr.body(), interner, indent_n + 1)
            ));
        }
        for element in self.elements.iter() {
            buf.push_str(&match element {
                ClassElement::MethodDefinition(name, method) => {
                    format!(
                        "{indentation}{}{}({}) {}\n",
                        match &method {
                            MethodDefinition::Get(_) => "get ",
                            MethodDefinition::Set(_) => "set ",
                            _ => "",
                        },
                        name.to_interned_string(interner),
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Generator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Async(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                ClassElement::StaticMethodDefinition(name, method) => {
                    format!(
                        "{indentation}static {}{}({}) {}\n",
                        match &method {
                            MethodDefinition::Get(_) => "get ",
                            MethodDefinition::Set(_) => "set ",
                            _ => "",
                        },
                        name.to_interned_string(interner),
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Generator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Async(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                ClassElement::FieldDefinition(name, field) => match field {
                    Some(expr) => {
                        format!(
                            "{indentation}{} = {};\n",
                            name.to_interned_string(interner),
                            expr.to_no_indent_string(interner, indent_n + 1)
                        )
                    }
                    None => {
                        format!("{indentation}{};\n", name.to_interned_string(interner),)
                    }
                },
                ClassElement::StaticFieldDefinition(name, field) => match field {
                    Some(expr) => {
                        format!(
                            "{indentation}static {} = {};\n",
                            name.to_interned_string(interner),
                            expr.to_no_indent_string(interner, indent_n + 1)
                        )
                    }
                    None => {
                        format!(
                            "{indentation}static {};\n",
                            name.to_interned_string(interner),
                        )
                    }
                },
                ClassElement::PrivateMethodDefinition(name, method) => {
                    format!(
                        "{indentation}{}#{}({}) {}\n",
                        match &method {
                            MethodDefinition::Get(_) => "get ",
                            MethodDefinition::Set(_) => "set ",
                            _ => "",
                        },
                        interner.resolve_expect(*name),
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Generator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Async(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                ClassElement::PrivateStaticMethodDefinition(name, method) => {
                    format!(
                        "{indentation}static {}#{}({}) {}\n",
                        match &method {
                            MethodDefinition::Get(_) => "get ",
                            MethodDefinition::Set(_) => "set ",
                            _ => "",
                        },
                        interner.resolve_expect(*name),
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Generator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                            MethodDefinition::Async(expr) => {
                                join_nodes(interner, &expr.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(expr)
                            | MethodDefinition::Set(expr)
                            | MethodDefinition::Ordinary(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(expr) => {
                                block_to_string(expr.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                ClassElement::PrivateFieldDefinition(name, field) => match field {
                    Some(expr) => {
                        format!(
                            "{indentation}#{} = {};\n",
                            interner.resolve_expect(*name),
                            expr.to_no_indent_string(interner, indent_n + 1)
                        )
                    }
                    None => {
                        format!("{indentation}#{};\n", interner.resolve_expect(*name),)
                    }
                },
                ClassElement::PrivateStaticFieldDefinition(name, field) => match field {
                    Some(expr) => {
                        format!(
                            "{indentation}static #{} = {};\n",
                            interner.resolve_expect(*name),
                            expr.to_no_indent_string(interner, indent_n + 1)
                        )
                    }
                    None => {
                        format!("{indentation}static #{};\n", interner.resolve_expect(*name),)
                    }
                },
                ClassElement::StaticBlock(statement_list) => {
                    format!(
                        "{indentation}static {}\n",
                        block_to_string(statement_list, interner, indent_n + 1)
                    )
                }
            });
        }
        buf.push('}');
        buf
    }
}

impl From<Class> for Expression {
    fn from(expr: Class) -> Self {
        Self::Class(Box::new(expr))
    }
}

impl From<Class> for Declaration {
    fn from(f: Class) -> Self {
        Self::Class(f)
    }
}

/// An element that can be within a [`Class`], as defined by the [spec].
///
/// [spec]: https://tc39.es/ecma262/#prod-ClassElement
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClassElement {
    /// A method definition, including `get` and `set` accessors.
    MethodDefinition(PropertyName, MethodDefinition),
    /// A static method definition, accessible from the class constructor object.
    StaticMethodDefinition(PropertyName, MethodDefinition),
    /// A field definition.
    FieldDefinition(PropertyName, Option<Expression>),
    /// A static field definition, accessible from the class constructor object
    StaticFieldDefinition(PropertyName, Option<Expression>),
    /// A private method definition, only accessible inside the class declaration.
    PrivateMethodDefinition(Sym, MethodDefinition),
    /// A private static method definition, only accessible from static methods and fields inside
    /// the class declaration.
    PrivateStaticMethodDefinition(Sym, MethodDefinition),
    /// A private field definition, only accessible inside the class declaration.
    PrivateFieldDefinition(Sym, Option<Expression>),
    /// A private static field definition, only accessible from static methods and fields inside the
    /// class declaration.
    PrivateStaticFieldDefinition(Sym, Option<Expression>),
    /// A static block, where a class can have initialization logic for its static fields.
    StaticBlock(StatementList),
}

impl ClassElement {
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            // Skipping function since they must not have names
            Self::MethodDefinition(name, _) | Self::StaticMethodDefinition(name, _) => {
                name.contains_arguments()
            }
            Self::FieldDefinition(name, Some(init))
            | Self::StaticFieldDefinition(name, Some(init)) => {
                name.contains_arguments() || init.contains_arguments()
            }
            Self::PrivateFieldDefinition(_, Some(init))
            | Self::PrivateStaticFieldDefinition(_, Some(init)) => init.contains_arguments(),
            Self::StaticBlock(statement_list) => statement_list
                .statements()
                .iter()
                .any(StatementListItem::contains_arguments),
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            // Skipping function since they must not have names
            Self::MethodDefinition(name, _) | Self::StaticMethodDefinition(name, _) => {
                name.contains(symbol)
            }
            Self::FieldDefinition(name, Some(init))
            | Self::StaticFieldDefinition(name, Some(init)) => {
                name.contains(symbol) || init.contains(symbol)
            }
            Self::PrivateFieldDefinition(_, Some(init))
            | Self::PrivateStaticFieldDefinition(_, Some(init)) => init.contains(symbol),
            Self::StaticBlock(_statement_list) => false,
            _ => false,
        }
    }
}

impl VisitWith for Class {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        if let Some(ident) = &self.name {
            try_break!(visitor.visit_identifier(ident));
        }
        if let Some(expr) = &self.super_ref {
            try_break!(visitor.visit_expression(expr));
        }
        if let Some(func) = &self.constructor {
            try_break!(visitor.visit_function(func));
        }
        for elem in self.elements.iter() {
            try_break!(visitor.visit_class_element(elem));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        if let Some(ident) = &mut self.name {
            try_break!(visitor.visit_identifier_mut(ident));
        }
        if let Some(expr) = &mut self.super_ref {
            try_break!(visitor.visit_expression_mut(expr));
        }
        if let Some(func) = &mut self.constructor {
            try_break!(visitor.visit_function_mut(func));
        }
        for elem in self.elements.iter_mut() {
            try_break!(visitor.visit_class_element_mut(elem));
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::ast::test_formatting;

    #[test]
    fn class_declaration_empty() {
        test_formatting(
            r#"
        class A {}
        "#,
        );
    }

    #[test]
    fn class_declaration_empty_extends() {
        test_formatting(
            r#"
        class A extends Object {}
        "#,
        );
    }

    #[test]
    fn class_declaration_constructor() {
        test_formatting(
            r#"
        class A {
            constructor(a, b, c) {
                this.value = a + b + c;
            }
        }
        "#,
        );
    }

    #[test]
    fn class_declaration_elements() {
        test_formatting(
            r#"
        class A {
            a;
            b = 1;
            c() {}
            d(a, b, c) {
                return a + b + c;
            }
            set e(value) {}
            get e() {}
            set(a, b) {}
            get(a, b) {}
        }
        "#,
        );
    }

    #[test]
    fn class_declaration_elements_private() {
        test_formatting(
            r#"
        class A {
            #a;
            #b = 1;
            #c() {}
            #d(a, b, c) {
                return a + b + c;
            }
            set #e(value) {}
            get #e() {}
        }
        "#,
        );
    }

    #[test]
    fn class_declaration_elements_static() {
        test_formatting(
            r#"
        class A {
            static a;
            static b = 1;
            static c() {}
            static d(a, b, c) {
                return a + b + c;
            }
            static set e(value) {}
            static get e() {}
        }
        "#,
        );
    }

    #[test]
    fn class_declaration_elements_private_static() {
        test_formatting(
            r#"
        class A {
            static #a;
            static #b = 1;
            static #c() {}
            static #d(a, b, c) {
                return a + b + c;
            }
            static set #e(value) {}
            static get #e() {}
        }
        "#,
        );
    }
}
