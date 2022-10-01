//! Object Expression.

#[cfg(test)]
mod tests;

use crate::syntax::ast::expression::Expression;
use crate::syntax::ast::ContainsSymbol;

use crate::syntax::ast::property::MethodDefinition;
use crate::syntax::ast::property::PropertyDefinition;
use crate::syntax::ast::{block_to_string, join_nodes};
use boa_interner::{Interner, ToInternedString};

/// Objects in JavaScript may be defined as an unordered collection of related data, of
/// primitive or reference types, in the form of “key: value” pairs.
///
/// Objects can be initialized using `new Object()`, `Object.create()`, or using the literal
/// notation.
///
/// An object initializer is an expression that describes the initialization of an
/// [`Object`][object]. Objects consist of properties, which are used to describe an object.
/// Values of object properties can either contain [`primitive`][primitive] data types or other
/// objects.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ObjectLiteral
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer
/// [object]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object
/// [primitive]: https://developer.mozilla.org/en-US/docs/Glossary/primitive
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "deser", serde(transparent))]
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectLiteral {
    properties: Box<[PropertyDefinition]>,
}

impl ObjectLiteral {
    pub fn properties(&self) -> &[PropertyDefinition] {
        &self.properties
    }

    /// Implements the display formatting with indentation.
    pub(crate) fn to_indented_string(&self, interner: &Interner, indent_n: usize) -> String {
        let mut buf = "{\n".to_owned();
        let indentation = "    ".repeat(indent_n + 1);
        for property in self.properties().iter() {
            buf.push_str(&match property {
                PropertyDefinition::IdentifierReference(ident) => {
                    format!("{indentation}{},\n", interner.resolve_expect(ident.sym()))
                }
                PropertyDefinition::Property(key, value) => {
                    format!(
                        "{indentation}{}: {},\n",
                        key.to_interned_string(interner),
                        value.to_no_indent_string(interner, indent_n + 1)
                    )
                }
                PropertyDefinition::SpreadObject(key) => {
                    format!("{indentation}...{},\n", key.to_interned_string(interner))
                }
                PropertyDefinition::MethodDefinition(key, method) => {
                    format!(
                        "{indentation}{}{}({}) {},\n",
                        match &method {
                            MethodDefinition::Get(_) => "get ",
                            MethodDefinition::Set(_) => "set ",
                            _ => "",
                        },
                        key.to_interned_string(interner),
                        match &method {
                            MethodDefinition::Get(expression)
                            | MethodDefinition::Set(expression)
                            | MethodDefinition::Ordinary(expression) => {
                                join_nodes(interner, &expression.parameters().parameters)
                            }
                            MethodDefinition::Generator(expression) => {
                                join_nodes(interner, &expression.parameters().parameters)
                            }
                            MethodDefinition::AsyncGenerator(expression) => {
                                join_nodes(interner, &expression.parameters().parameters)
                            }
                            MethodDefinition::Async(expression) => {
                                join_nodes(interner, &expression.parameters().parameters)
                            }
                        },
                        match &method {
                            MethodDefinition::Get(expression)
                            | MethodDefinition::Set(expression)
                            | MethodDefinition::Ordinary(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Generator(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::AsyncGenerator(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                            MethodDefinition::Async(expression) => {
                                block_to_string(expression.body(), interner, indent_n + 1)
                            }
                        },
                    )
                }
                PropertyDefinition::CoverInitializedName(ident, expr) => {
                    format!(
                        "{indentation}{} = {},\n",
                        interner.resolve_expect(ident.sym()),
                        expr.to_no_indent_string(interner, indent_n + 1)
                    )
                }
            });
        }
        buf.push_str(&format!("{}}}", "    ".repeat(indent_n)));

        buf
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.properties
            .iter()
            .any(PropertyDefinition::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.properties.iter().any(|prop| prop.contains(symbol))
    }
}

impl ToInternedString for ObjectLiteral {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl<T> From<T> for ObjectLiteral
where
    T: Into<Box<[PropertyDefinition]>>,
{
    fn from(props: T) -> Self {
        Self {
            properties: props.into(),
        }
    }
}

impl From<ObjectLiteral> for Expression {
    fn from(obj: ObjectLiteral) -> Self {
        Self::ObjectLiteral(obj)
    }
}
