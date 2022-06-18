use crate::syntax::ast::node::Node;
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// This property accessor provides access to an class object's private fields.
///
/// This expression can be described as ` MemberExpression.PrivateIdentifier`
/// Example: `this.#a`
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-MemberExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_class_fields
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct GetPrivateField {
    obj: Box<Node>,
    field: Sym,
}

impl GetPrivateField {
    /// Creates a `GetPrivateField` AST node.
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

impl ToInternedString for GetPrivateField {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}.#{}",
            self.obj.to_interned_string(interner),
            interner.resolve_expect(self.field)
        )
    }
}

impl From<GetPrivateField> for Node {
    fn from(get_private_field: GetPrivateField) -> Self {
        Self::GetPrivateField(get_private_field)
    }
}
