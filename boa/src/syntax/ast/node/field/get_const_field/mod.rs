use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// This property accessor provides access to an object's properties by using the
/// [dot notation][mdn].
///
/// In the object.property syntax, the property must be a valid JavaScript identifier.
/// (In the ECMAScript standard, the names of properties are technically "IdentifierNames", not
/// "Identifiers", so reserved words can be used but are not recommended).
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
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_accessors#Dot_notation
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct GetConstField {
    obj: Box<Node>,
    field: Sym,
}

impl GetConstField {
    /// Creates a `GetConstField` AST node.
    pub fn new<V>(value: V, field: Sym) -> Self
    where
        V: Into<Node>,
    {
        Self {
            obj: Box::new(value.into()),
            field,
        }
    }

    /// Gets the original object from where to get the field from.
    pub fn obj(&self) -> &Node {
        &self.obj
    }

    /// Gets the name of the field to retrieve.
    pub fn field(&self) -> Sym {
        self.field
    }
}

impl ToInternedString for GetConstField {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}.{}",
            self.obj.to_interned_string(interner),
            interner.resolve_expect(self.field)
        )
    }
}

impl From<GetConstField> for Node {
    fn from(get_const_field: GetConstField) -> Self {
        Self::GetConstField(get_const_field)
    }
}
