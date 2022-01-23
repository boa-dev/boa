//! Object node.

use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::{
        declaration::block_to_string, join_nodes, MethodDefinitionKind, Node, PropertyDefinition,
    },
};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "deser", serde(transparent))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Object {
    properties: Box<[PropertyDefinition]>,
}

impl Object {
    pub fn properties(&self) -> &[PropertyDefinition] {
        &self.properties
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn to_indented_string(
        &self,
        interner: &Interner,
        indent: usize,
    ) -> String {
        let mut buf = "{\n".to_owned();
        let indentation = "    ".repeat(indent + 1);
        for property in self.properties().iter() {
            buf.push_str(&match property {
                PropertyDefinition::IdentifierReference(key) => {
                    format!("{}{},\n", indentation, key)
                }
                PropertyDefinition::Property(key, value) => {
                    format!(
                        "{}{}: {},\n",
                        indentation,
                        key.to_interned_string(interner),
                        value.to_no_indent_string(interner, indent + 1)
                    )
                }
                PropertyDefinition::SpreadObject(key) => {
                    format!("{}...{},\n", indentation, key.to_interned_string(interner))
                }
                PropertyDefinition::MethodDefinition(kind, key, node) => {
                    format!(
                        "{}{}{}({}) {},\n",
                        indentation,
                        match &kind {
                            MethodDefinitionKind::Get => "get ",
                            MethodDefinitionKind::Set => "set ",
                            MethodDefinitionKind::Ordinary
                            | MethodDefinitionKind::Generator
                            | MethodDefinitionKind::Async
                            | MethodDefinitionKind::AsyncGenerator => "",
                        },
                        key.to_interned_string(interner),
                        join_nodes(interner, node.parameters()),
                        block_to_string(node.body(), interner, indent + 1)
                    )
                }
            });
        }
        buf.push_str(&format!("{}}}", "    ".repeat(indent)));

        buf
    }
}

impl ToInternedString for Object {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl<T> From<T> for Object
where
    T: Into<Box<[PropertyDefinition]>>,
{
    fn from(props: T) -> Self {
        Self {
            properties: props.into(),
        }
    }
}

impl From<Object> for Node {
    fn from(obj: Object) -> Self {
        Self::Object(obj)
    }
}
