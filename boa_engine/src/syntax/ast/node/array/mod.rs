//! Array declaration node.

use super::{join_nodes, Node};
use boa_interner::{Interner, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// An array is an ordered collection of data (either primitive or object depending upon the
/// language).
///
/// Arrays are used to store multiple values in a single variable.
/// This is compared to a variable that can store only one value.
///
/// Each item in an array has a number attached to it, called a numeric index, that allows you
/// to access it. In JavaScript, arrays start at index zero and can be manipulated with various
/// methods.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ArrayLiteral
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayDecl {
    arr: Box<[Node]>,
    has_trailing_comma_spread: bool,
}

impl ArrayDecl {
    /// Crate a new array declaration.
    pub(crate) fn new<A>(array: A, has_trailing_comma_spread: bool) -> Self
    where
        A: Into<Box<[Node]>>,
    {
        Self {
            arr: array.into(),
            has_trailing_comma_spread,
        }
    }

    /// Indicates if a spread operator in the array literal has a trailing comma.
    /// This is a syntax error in some cases.
    pub(crate) fn has_trailing_comma_spread(&self) -> bool {
        self.has_trailing_comma_spread
    }
}

impl AsRef<[Node]> for ArrayDecl {
    fn as_ref(&self) -> &[Node] {
        &self.arr
    }
}

impl<T> From<T> for ArrayDecl
where
    T: Into<Box<[Node]>>,
{
    fn from(decl: T) -> Self {
        Self {
            arr: decl.into(),
            has_trailing_comma_spread: false,
        }
    }
}

impl ToInternedString for ArrayDecl {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("[{}]", join_nodes(interner, &self.arr))
    }
}

impl From<ArrayDecl> for Node {
    fn from(arr: ArrayDecl) -> Self {
        Self::ArrayDecl(arr)
    }
}
