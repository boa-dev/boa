//! Update expression nodes.
//!
//! A update expression increments or decrements it's operand and returns a value
//!
//! - [Increment and decrement operations][mdn] (`++`, `--`).
//!
//! The full list of valid update operators is defined in [`UpdateOp`].
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#increment_and_decrement
mod op;

use crate::{
    Expression, Span, Spanned,
    expression::{Identifier, access::PropertyAccess},
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

pub use op::*;

/// A update expression is an operation with only one operand.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UpdateExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#increment_and_decrement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Update<'arena> {
    op: UpdateOp,
    target: Box<UpdateTarget<'arena>>,
    span: Span,
}

impl<'arena> Update<'arena> {
    /// Creates a new `Update` AST expression.
    #[inline]
    #[must_use]
    pub fn new(op: UpdateOp, target: UpdateTarget<'arena>, span: Span) -> Self {
        Self {
            op,
            target: Box::new(target),
            span,
        }
    }

    /// Gets the update operation of the expression.
    #[inline]
    #[must_use]
    pub const fn op(&self) -> UpdateOp {
        self.op
    }

    /// Gets the target of this update operator.
    #[inline]
    #[must_use]
    pub fn target(&self) -> &UpdateTarget<'arena> {
        self.target.as_ref()
    }
}

impl<'arena> Spanned for Update<'arena> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> ToInternedString for Update<'arena> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self.op {
            UpdateOp::IncrementPost | UpdateOp::DecrementPost => {
                format!("{}{}", self.target.to_interned_string(interner), self.op)
            }
            UpdateOp::IncrementPre | UpdateOp::DecrementPre => {
                format!("{}{}", self.op, self.target.to_interned_string(interner))
            }
        }
    }
}

impl<'arena> From<Update<'arena>> for Expression<'arena> {
    #[inline]
    fn from(op: Update<'arena>) -> Self {
        Self::Update(op)
    }
}

impl<'arena> VisitWith<'arena> for Update<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        match self.target.as_ref() {
            UpdateTarget::Identifier(ident) => visitor.visit_identifier(ident),
            UpdateTarget::PropertyAccess(access) => visitor.visit_property_access(access),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        match &mut *self.target {
            UpdateTarget::Identifier(ident) => visitor.visit_identifier_mut(ident),
            UpdateTarget::PropertyAccess(access) => visitor.visit_property_access_mut(access),
        }
    }
}

/// A update expression can only be performed on identifier expressions or property access expressions.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-UpdateExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#increment_and_decrement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum UpdateTarget<'arena> {
    /// An [`Identifier`] expression.
    Identifier(Identifier),

    /// An [`PropertyAccess`] expression.
    PropertyAccess(PropertyAccess<'arena>),
}

impl<'arena> ToInternedString for UpdateTarget<'arena> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Identifier(identifier) => identifier.to_interned_string(interner),
            Self::PropertyAccess(access) => access.to_interned_string(interner),
        }
    }
}
