use std::borrow::Cow;

use crate::string::ToStringEscaped;
use crate::syntax::ast::expression::Expression;
use crate::syntax::ast::property::{MethodDefinition, PropertyName};
use crate::syntax::ast::statement::StatementList;
use crate::syntax::ast::{block_to_string, join_nodes, ContainsSymbol, Statement};
use boa_interner::{Interner, Sym, ToInternedString};

use super::Function;

/// The `class` declaration defines a class with the specified methods, fields, and optional constructor.
///
/// Classes can be used to create objects, which can also be created through literals (using `{}`).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-class-definitions
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/function
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    name: Option<Sym>,
    super_ref: Option<Expression>,
    constructor: Option<Function>,
    elements: Box<[ClassElement]>,
}

impl Class {
    /// Creates a new class declaration.
    pub(in crate::syntax) fn new(
        name: Option<Sym>,
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
    pub(crate) fn name(&self) -> Option<Sym> {
        self.name
    }

    /// Returns the super class ref of the class.
    pub(crate) fn super_ref(&self) -> Option<&Expression> {
        self.super_ref.as_ref()
    }

    /// Returns the constructor of the class.
    pub(crate) fn constructor(&self) -> Option<&Function> {
        self.constructor.as_ref()
    }

    /// Gets the list of all fields defined on the class.
    pub(crate) fn elements(&self) -> &[ClassElement] {
        &self.elements
    }

    /// Implements the display formatting with indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let class_name = self.name.map_or(Cow::Borrowed(""), |s| {
            interner.resolve_expect(s).join(
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

impl ToInternedString for Class {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Class> for Expression {
    fn from(expr: Class) -> Self {
        Self::Class(Box::new(expr))
    }
}

impl From<Class> for Statement {
    fn from(f: Class) -> Self {
        Self::Class(f)
    }
}

/// Class element types.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClassElement {
    MethodDefinition(PropertyName, MethodDefinition),
    StaticMethodDefinition(PropertyName, MethodDefinition),
    FieldDefinition(PropertyName, Option<Expression>),
    StaticFieldDefinition(PropertyName, Option<Expression>),
    PrivateMethodDefinition(Sym, MethodDefinition),
    PrivateStaticMethodDefinition(Sym, MethodDefinition),
    PrivateFieldDefinition(Sym, Option<Expression>),
    PrivateStaticFieldDefinition(Sym, Option<Expression>),
    StaticBlock(StatementList),
}

impl ClassElement {
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Self::MethodDefinition(_, method) | Self::StaticMethodDefinition(_, method) => {
                match method {
                    MethodDefinition::Get(function)
                    | MethodDefinition::Set(function)
                    | MethodDefinition::Ordinary(function) => {
                        matches!(function.name(), Some(Sym::ARGUMENTS))
                    }
                    MethodDefinition::Generator(generator) => {
                        matches!(generator.name(), Some(Sym::ARGUMENTS))
                    }
                    MethodDefinition::AsyncGenerator(async_generator) => {
                        matches!(async_generator.name(), Some(Sym::ARGUMENTS))
                    }
                    MethodDefinition::Async(function) => {
                        matches!(function.name(), Some(Sym::ARGUMENTS))
                    }
                }
            }
            Self::FieldDefinition(_, Some(node))
            | Self::StaticFieldDefinition(_, Some(node))
            | Self::PrivateFieldDefinition(_, Some(node))
            | Self::PrivateStaticFieldDefinition(_, Some(node)) => node.contains_arguments(),
            Self::StaticBlock(statement_list) => statement_list
                .statements()
                .iter()
                .any(Statement::contains_arguments),
            _ => false,
        }
    }

    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::MethodDefinition(name, _)
            | Self::StaticMethodDefinition(name, _)
            | Self::FieldDefinition(name, _)
            | Self::StaticFieldDefinition(name, _) => {
                matches!(name.computed(), Some(expr) if expr.contains(symbol))
            }
            _ => false,
        }
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
