//! Object node.

use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    property::{AccessorDescriptor, Attribute, DataDescriptor, PropertyDescriptor},
    syntax::ast::node::{join_nodes, MethodDefinitionKind, Node, PropertyDefinition},
    BoaProfiler, Context, Result, Value,
};
use std::fmt;

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
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
    ) -> fmt::Result {
        f.write_str("{\n")?;
        let indentation = "    ".repeat(indent + 1);
        for property in self.properties().iter() {
            match property {
                PropertyDefinition::IdentifierReference(key) => {
                    writeln!(f, "{}{},", indentation, key)?;
                }
                PropertyDefinition::Property(key, value) => {
                    write!(f, "{}{}: ", indentation, key,)?;
                    value.display_no_indent(f, indent + 1)?;
                    writeln!(f, ",")?;
                }
                PropertyDefinition::SpreadObject(key) => {
                    writeln!(f, "{}...{},", indentation, key)?;
                }
                PropertyDefinition::MethodDefinition(kind, key, node) => {
                    write!(f, "{}", indentation)?;
                    match &kind {
                        MethodDefinitionKind::Get => write!(f, "get ")?,
                        MethodDefinitionKind::Set => write!(f, "set ")?,
                        MethodDefinitionKind::Ordinary => (),
                    }
                    write!(f, "{}(", key)?;
                    join_nodes(f, &node.parameters())?;
                    write!(f, ") ")?;
                    node.display_block(f, indent + 1)?;
                    writeln!(f, ",")?;
                }
            }
        }
        write!(f, "{}}}", "    ".repeat(indent))
    }
}

impl Executable for Object {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("object", "exec");
        let obj = Value::new_object(context);

        // TODO: Implement the rest of the property types.
        for property in self.properties().iter() {
            match property {
                PropertyDefinition::Property(key, value) => {
                    obj.set_property(
                        key.clone(),
                        PropertyDescriptor::Data(DataDescriptor::new(
                            value.run(context)?,
                            Attribute::all(),
                        )),
                    );
                }
                PropertyDefinition::MethodDefinition(kind, name, func) => match kind {
                    MethodDefinitionKind::Ordinary => {
                        obj.set_property(
                            name.clone(),
                            PropertyDescriptor::Data(DataDescriptor::new(
                                func.run(context)?,
                                Attribute::all(),
                            )),
                        );
                    }
                    MethodDefinitionKind::Get => {
                        let set = obj
                            .get_property(name.clone())
                            .as_ref()
                            .and_then(|p| p.as_accessor_descriptor())
                            .and_then(|a| a.setter().cloned());
                        obj.set_property(
                            name.clone(),
                            PropertyDescriptor::Accessor(AccessorDescriptor {
                                get: func.run(context)?.as_object(),
                                set,
                                attributes: Attribute::WRITABLE
                                    | Attribute::ENUMERABLE
                                    | Attribute::CONFIGURABLE,
                            }),
                        )
                    }
                    MethodDefinitionKind::Set => {
                        let get = obj
                            .get_property(name.clone())
                            .as_ref()
                            .and_then(|p| p.as_accessor_descriptor())
                            .and_then(|a| a.getter().cloned());
                        obj.set_property(
                            name.clone(),
                            PropertyDescriptor::Accessor(AccessorDescriptor {
                                get,
                                set: func.run(context)?.as_object(),
                                attributes: Attribute::WRITABLE
                                    | Attribute::ENUMERABLE
                                    | Attribute::CONFIGURABLE,
                            }),
                        )
                    }
                },
                _ => {} //unimplemented!("{:?} type of property", i),
            }
        }

        Ok(obj)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
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
