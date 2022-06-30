use crate::syntax::ast::node::Node;
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// The `super` keyword is used to access fields on an object's parent.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SuperProperty
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/super
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum GetSuperField {
    Const(Sym),
    Expr(Box<Node>),
}

impl From<Sym> for GetSuperField {
    fn from(field: Sym) -> Self {
        Self::Const(field)
    }
}

impl From<Node> for GetSuperField {
    fn from(field: Node) -> Self {
        Self::Expr(Box::new(field))
    }
}

impl ToInternedString for GetSuperField {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            GetSuperField::Const(field) => format!("super.{}", interner.resolve_expect(*field)),
            GetSuperField::Expr(field) => format!("super[{}]", field.to_interned_string(interner)),
        }
    }
}

impl From<GetSuperField> for Node {
    fn from(get_super_field: GetSuperField) -> Self {
        Self::GetSuperField(get_super_field)
    }
}
