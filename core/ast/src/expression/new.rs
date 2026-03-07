use crate::expression::Call;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{Span, Spanned};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct New<'arena> {
    call: Call<'arena>,
}

impl<'arena> New<'arena> {
    /// Gets the constructor of the new expression.
    #[inline]
    #[must_use]
    pub const fn constructor(&self) -> &Expression<'arena> {
        self.call.function()
    }

    /// Retrieves the arguments passed to the constructor.
    #[inline]
    #[must_use]
    pub const fn arguments(&self) -> &[Expression<'arena>] {
        self.call.args()
    }

    /// Returns the inner call expression.
    #[must_use]
    pub const fn call(&self) -> &Call<'arena> {
        &self.call
    }
}

impl<'arena> From<Call<'arena>> for New<'arena> {
    #[inline]
    fn from(call: Call<'arena>) -> Self {
        Self { call }
    }
}

impl Spanned for New<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.call.span()
    }
}

impl ToInternedString for New<'_> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("new {}", self.call.to_interned_string(interner))
    }
}

impl<'arena> From<New<'arena>> for Expression<'arena> {
    #[inline]
    fn from(new: New<'arena>) -> Self {
        Self::New(new)
    }
}

impl<'arena> VisitWith<'arena> for New<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_call(&self.call)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_call_mut(&mut self.call)
    }
}
