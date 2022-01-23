use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// This property accessor provides access to an object's properties by using the
/// [bracket notation][mdn].
///
/// In the object\[property_name\] syntax, the property_name is just a string or
/// [Symbol][symbol]. So, it can be any string, including '1foo', '!bar!', or even ' ' (a
/// space).
///
/// One can think of an object as an associative array (a.k.a. map, dictionary, hash, lookup
/// table). The keys in this array are the names of the object's properties.
///
/// It's typical when speaking of an object's properties to make a distinction between
/// properties and methods. However, the property/method distinction is little more than a
/// convention. A method is simply a property that can be called (for example, if it has a
/// reference to a Function instance as its value).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-property-accessors
/// [symbol]: https://developer.mozilla.org/en-US/docs/Glossary/Symbol
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_accessors#Bracket_notation
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct GetField {
    obj: Box<Node>,
    field: Box<Node>,
}

impl GetField {
    pub fn obj(&self) -> &Node {
        &self.obj
    }

    pub fn field(&self) -> &Node {
        &self.field
    }

    /// Creates a `GetField` AST node.
    pub fn new<V, F>(value: V, field: F) -> Self
    where
        V: Into<Node>,
        F: Into<Node>,
    {
        Self {
            obj: Box::new(value.into()),
            field: Box::new(field.into()),
        }
    }
}

impl ToInternedString for GetField {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}[{}]",
            self.obj.to_interned_string(interner),
            self.field.to_interned_string(interner)
        )
    }
}

impl From<GetField> for Node {
    fn from(get_field: GetField) -> Self {
        Self::GetField(get_field)
    }
}
