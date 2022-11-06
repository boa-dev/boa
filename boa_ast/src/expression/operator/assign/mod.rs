//! Assignment expression nodes, as defined by the [spec].
//!
//! An [assignment operator][mdn] assigns a value to its left operand based on the value of its right
//! operand. Almost any [`LeftHandSideExpression`][lhs] Parse Node can be the target of a simple
//! assignment expression (`=`). However, the compound assignment operations such as `%=` or `??=`
//! only allow ["simple"][simple] left hand side expressions as an assignment target.
//!
//! [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment_Operators
//! [lhs]: https://tc39.es/ecma262/#prod-LeftHandSideExpression
//! [simple]: https://tc39.es/ecma262/#sec-static-semantics-assignmenttargettype

mod op;

use core::ops::ControlFlow;
pub use op::*;

use boa_interner::{Interner, ToInternedString};

use crate::{
    expression::{access::PropertyAccess, identifier::Identifier, Expression},
    pattern::Pattern,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
};

/// An assignment operator expression.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Assign {
    op: AssignOp,
    lhs: Box<AssignTarget>,
    rhs: Box<Expression>,
}

impl Assign {
    /// Creates an `Assign` AST Expression.
    #[must_use]
    pub fn new(op: AssignOp, lhs: AssignTarget, rhs: Expression) -> Self {
        Self {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Gets the operator of the assignment operation.
    #[inline]
    #[must_use]
    pub fn op(&self) -> AssignOp {
        self.op
    }

    /// Gets the left hand side of the assignment operation.
    #[inline]
    #[must_use]
    pub fn lhs(&self) -> &AssignTarget {
        &self.lhs
    }

    /// Gets the right hand side of the assignment operation.
    #[inline]
    #[must_use]
    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
}

impl ToInternedString for Assign {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{} {} {}",
            self.lhs.to_interned_string(interner),
            self.op,
            self.rhs.to_interned_string(interner)
        )
    }
}

impl From<Assign> for Expression {
    #[inline]
    fn from(op: Assign) -> Self {
        Self::Assign(op)
    }
}

impl VisitWith for Assign {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_assign_target(&self.lhs));
        visitor.visit_expression(&self.rhs)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_assign_target_mut(&mut self.lhs));
        visitor.visit_expression_mut(&mut self.rhs)
    }
}

/// The valid left-hand-side expressions of an assignment operator. Also called
/// [`LeftHandSideExpression`][spec] in the spec.
///
/// [spec]: hhttps://tc39.es/ecma262/#prod-LeftHandSideExpression
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum AssignTarget {
    /// A simple identifier, such as `a`.
    Identifier(Identifier),
    /// A property access, such as `a.prop`.
    Access(PropertyAccess),
    /// A pattern assignment, such as `{a, b, ...c}`.
    Pattern(Pattern),
}

impl AssignTarget {
    /// Converts the left-hand-side Expression of an assignment expression into an [`AssignTarget`].
    /// Returns `None` if the given Expression is an invalid left-hand-side for a assignment expression.
    #[must_use]
    pub fn from_expression(
        expression: &Expression,
        strict: bool,
        destructure: bool,
    ) -> Option<Self> {
        match expression {
            Expression::Identifier(id) => Some(Self::Identifier(*id)),
            Expression::PropertyAccess(access) => Some(Self::Access(access.clone())),
            Expression::ObjectLiteral(object) if destructure => {
                let pattern = object.to_pattern(strict)?;
                Some(Self::Pattern(pattern.into()))
            }
            Expression::ArrayLiteral(array) if destructure => {
                let pattern = array.to_pattern(strict)?;
                Some(Self::Pattern(pattern.into()))
            }
            _ => None,
        }
    }
}

impl ToInternedString for AssignTarget {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            AssignTarget::Identifier(id) => id.to_interned_string(interner),
            AssignTarget::Access(access) => access.to_interned_string(interner),
            AssignTarget::Pattern(pattern) => pattern.to_interned_string(interner),
        }
    }
}

impl From<Identifier> for AssignTarget {
    #[inline]
    fn from(target: Identifier) -> Self {
        Self::Identifier(target)
    }
}

impl VisitWith for AssignTarget {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            AssignTarget::Identifier(id) => visitor.visit_identifier(id),
            AssignTarget::Access(pa) => visitor.visit_property_access(pa),
            AssignTarget::Pattern(pat) => visitor.visit_pattern(pat),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            AssignTarget::Identifier(id) => visitor.visit_identifier_mut(id),
            AssignTarget::Access(pa) => visitor.visit_property_access_mut(pa),
            AssignTarget::Pattern(pat) => visitor.visit_pattern_mut(pat),
        }
    }
}
