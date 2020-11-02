//! Object node.

use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::{MethodDefinitionKind, Node, PropertyDefinition},
    Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
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
        for property in self.properties().iter() {
            match property {
                PropertyDefinition::IdentifierReference(key) => {
                    write!(f, "{}    {},", indent, key)?;
                }
                PropertyDefinition::Property(key, value) => {
                    write!(f, "{}    {}: {},", indent, key, value)?;
                }
                PropertyDefinition::SpreadObject(key) => {
                    write!(f, "{}    ...{},", indent, key)?;
                }
                PropertyDefinition::MethodDefinition(_kind, _key, _node) => {
                    // TODO: Implement display for PropertyDefinition::MethodDefinition.
                    unimplemented!("Display for PropertyDefinition::MethodDefinition");
                }
            }
        }
        f.write_str("}")
    }
}

impl Executable for Object {
    fn run(&self, context: &mut Context) -> Result<Value> {
        let global_val = &context
            .realm()
            .environment
            .get_global_object()
            .expect("Could not get the global object");
        let obj = Value::new_object(Some(global_val));

        // TODO: Implement the rest of the property types.
        for property in self.properties().iter() {
            match property {
                PropertyDefinition::Property(key, value) => {
                    obj.set_field(key.clone(), value.run(context)?);
                }
                PropertyDefinition::MethodDefinition(kind, name, func) => {
                    if let MethodDefinitionKind::Ordinary = kind {
                        obj.set_field(name.clone(), func.run(context)?);
                    } else {
                        // TODO: Implement other types of MethodDefinitionKinds.
                        //unimplemented!("other types of property method definitions.");
                    }
                }
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
