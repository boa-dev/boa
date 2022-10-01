use crate::syntax::ast::{expression::Call, ContainsSymbol};
use boa_interner::{Interner, ToInternedString};

use super::Expression;

/// The `new` operator lets developers create an instance of a user-defined object type or of
/// one of the built-in object types that has a constructor function.
///
/// The new keyword does the following things:
///  - Creates a blank, plain JavaScript object;
///  - Links (sets the constructor of) this object to another object;
///  - Passes the newly created object from Step 1 as the this context;
///  - Returns this if the function doesn't return its own object.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-NewExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct New {
    call: Call,
}

impl New {
    /// Gets the name of the function call.
    pub fn expr(&self) -> &Expression {
        self.call.expr()
    }

    /// Retrieves the arguments passed to the function.
    pub fn args(&self) -> &[Expression] {
        self.call.args()
    }

    /// Returns the inner call
    pub(crate) fn call(&self) -> &Call {
        &self.call
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.call.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.call.contains(symbol)
    }
}

impl From<Call> for New {
    fn from(call: Call) -> Self {
        Self { call }
    }
}

impl ToInternedString for New {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("new {}", self.call.to_interned_string(interner))
    }
}

impl From<New> for Expression {
    fn from(new: New) -> Self {
        Self::New(new)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fmt() {
        crate::syntax::ast::test_formatting(
            r#"
        function MyClass() {}
        let inst = new MyClass();
        "#,
        );
    }
}
