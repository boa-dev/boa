#[cfg(test)]
mod tests;

use crate::syntax::ast::node::{
    declaration::{block_to_string, FunctionExpr},
    join_nodes,
    object::{MethodDefinition, PropertyName},
    Node, StatementList,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    name: Sym,
    super_ref: Option<Box<Node>>,
    constructor: Option<FunctionExpr>,
    elements: Box<[ClassElement]>,
}

impl Class {
    /// Creates a new class declaration.
    pub(in crate::syntax) fn new<S, C, E>(
        name: Sym,
        super_ref: S,
        constructor: C,
        elements: E,
    ) -> Self
    where
        S: Into<Option<Box<Node>>>,
        C: Into<Option<FunctionExpr>>,
        E: Into<Box<[ClassElement]>>,
    {
        Self {
            name,
            super_ref: super_ref.into(),
            constructor: constructor.into(),
            elements: elements.into(),
        }
    }

    /// Returns the name of the class.
    pub(crate) fn name(&self) -> Sym {
        self.name
    }

    /// Returns the super class ref of the class.
    pub(crate) fn super_ref(&self) -> &Option<Box<Node>> {
        &self.super_ref
    }

    /// Returns the constructor of the class.
    pub(crate) fn constructor(&self) -> &Option<FunctionExpr> {
        &self.constructor
    }

    /// Gets the list of all fields defined on the class.
    pub(crate) fn elements(&self) -> &[ClassElement] {
        &self.elements
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indent_n: usize,
    ) -> String {
        if self.elements.is_empty() && self.constructor().is_none() {
            return format!(
                "class {}{} {{}}",
                interner.resolve_expect(self.name),
                if let Some(node) = &self.super_ref {
                    format!(" extends {}", node.to_interned_string(interner))
                } else {
                    "".to_string()
                }
            );
        }
        let indentation = "    ".repeat(indent_n + 1);
        let mut buf = format!(
            "class {}{} {{\n",
            interner.resolve_expect(self.name),
            if let Some(node) = &self.super_ref {
                format!("extends {}", node.to_interned_string(interner))
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
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Generator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Async(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
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
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Generator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Async(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                ClassElement::FieldDefinition(name, field) => match field {
                    Some(node) => {
                        format!(
                            "{indentation}{} = {};\n",
                            name.to_interned_string(interner),
                            node.to_no_indent_string(interner, indent_n + 1)
                        )
                    }
                    None => {
                        format!("{indentation}{};\n", name.to_interned_string(interner),)
                    }
                },
                ClassElement::StaticFieldDefinition(name, field) => match field {
                    Some(node) => {
                        format!(
                            "{indentation}static {} = {};\n",
                            name.to_interned_string(interner),
                            node.to_no_indent_string(interner, indent_n + 1)
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
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Generator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Async(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
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
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Generator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                            MethodDefinition::Async(node) => {
                                join_nodes(interner, &node.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(node)
                            | MethodDefinition::Set(node)
                            | MethodDefinition::Ordinary(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(node) => {
                                block_to_string(node.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                ClassElement::PrivateFieldDefinition(name, field) => match field {
                    Some(node) => {
                        format!(
                            "{indentation}#{} = {};\n",
                            interner.resolve_expect(*name),
                            node.to_no_indent_string(interner, indent_n + 1)
                        )
                    }
                    None => {
                        format!("{indentation}#{};\n", interner.resolve_expect(*name),)
                    }
                },
                ClassElement::PrivateStaticFieldDefinition(name, field) => match field {
                    Some(node) => {
                        format!(
                            "{indentation}static #{} = {};\n",
                            interner.resolve_expect(*name),
                            node.to_no_indent_string(interner, indent_n + 1)
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

/// Class element types.
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum ClassElement {
    MethodDefinition(PropertyName, MethodDefinition),
    StaticMethodDefinition(PropertyName, MethodDefinition),
    FieldDefinition(PropertyName, Option<Node>),
    StaticFieldDefinition(PropertyName, Option<Node>),
    PrivateMethodDefinition(Sym, MethodDefinition),
    PrivateStaticMethodDefinition(Sym, MethodDefinition),
    PrivateFieldDefinition(Sym, Option<Node>),
    PrivateStaticFieldDefinition(Sym, Option<Node>),
    StaticBlock(StatementList),
}
