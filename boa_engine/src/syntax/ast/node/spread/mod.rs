use crate::syntax::ast::node::Node;
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// The `spread` operator allows an iterable such as an array expression or string to be
/// expanded.
///
/// Syntax: `...x`
///
/// It expands array expressions or strings in places where zero or more arguments (for
/// function calls) or elements (for array literals)
/// are expected, or an object expression to be expanded in places where zero or more key-value
/// pairs (for object literals) are expected.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SpreadElement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "deser", serde(transparent))]
#[derive(Clone, Debug, PartialEq)]
pub struct Spread {
    val: Box<Node>,
}

impl Spread {
    pub fn val(&self) -> &Node {
        &self.val
    }

    /// Creates a `Spread` AST node.
    pub fn new<V>(val: V) -> Self
    where
        V: Into<Node>,
    {
        Self {
            val: Box::new(val.into()),
        }
    }
}

impl ToInternedString for Spread {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("...{}", self.val().to_interned_string(interner))
    }
}

impl From<Spread> for Node {
    fn from(spread: Spread) -> Self {
        Self::Spread(spread)
    }
}
