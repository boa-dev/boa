#![allow(clippy::doc_link_with_quotes)]

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

use boa_interner::{Interner, Sym, ToInternedString};

use crate::{
    Span, Spanned,
    expression::{Expression, access::PropertyAccess, identifier::Identifier},
    pattern::Pattern,
    visitor::{VisitWith, Visitor, VisitorMut},
};

/// An assignment operator expression.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Assign<'arena> {
    op: AssignOp,
    lhs: Box<AssignTarget<'arena>>,
    rhs: Box<Expression<'arena>>,
}

impl<'arena> Assign<'arena> {
    /// Creates an `Assign` AST Expression.
    #[inline]
    #[must_use]
    pub fn new(op: AssignOp, lhs: AssignTarget<'arena>, rhs: Expression<'arena>) -> Self {
        Self {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Gets the operator of the assignment operation.
    #[inline]
    #[must_use]
    pub const fn op(&self) -> AssignOp {
        self.op
    }

    /// Gets the left hand side of the assignment operation.
    #[inline]
    #[must_use]
    pub const fn lhs(&self) -> &AssignTarget<'arena> {
        &self.lhs
    }

    /// Gets the right hand side of the assignment operation.
    #[inline]
    #[must_use]
    pub const fn rhs(&self) -> &Expression<'arena> {
        &self.rhs
    }
}

impl<'arena> Spanned for Assign<'arena> {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.lhs.span().start(), self.rhs.span().end())
    }
}

impl<'arena> ToInternedString for Assign<'arena> {
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

impl<'arena> From<Assign<'arena>> for Expression<'arena> {
    #[inline]
    fn from(op: Assign<'arena>) -> Self {
        Self::Assign(op)
    }
}

impl<'arena> VisitWith<'arena> for Assign<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_assign_target(&self.lhs)?;
        visitor.visit_expression(&self.rhs)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_assign_target_mut(&mut self.lhs)?;
        visitor.visit_expression_mut(&mut self.rhs)
    }
}

/// The valid left-hand-side expressions of an assignment operator. Also called
/// [`LeftHandSideExpression`][spec] in the spec.
///
/// [spec]: hhttps://tc39.es/ecma262/#prod-LeftHandSideExpression
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum AssignTarget<'arena> {
    /// A simple identifier, such as `a`.
    Identifier(Identifier),
    /// A property access, such as `a.prop`.
    Access(PropertyAccess<'arena>),
    /// A pattern assignment, such as `{a, b, ...c}`.
    Pattern(Pattern<'arena>),
}

impl<'arena> AssignTarget<'arena> {
    /// Converts the left-hand-side Expression of an assignment expression into an [`AssignTarget`].
    /// Returns `None` if the given Expression is an invalid left-hand-side for a assignment expression.
    #[must_use]
    pub fn from_expression(expression: &Expression<'arena>, strict: bool) -> Option<Self> {
        match expression {
            Expression::ObjectLiteral(object) => {
                let pattern = object.to_pattern(strict)?;
                Some(Self::Pattern(pattern.into()))
            }
            Expression::ArrayLiteral(array) => {
                let pattern = array.to_pattern(strict)?;
                Some(Self::Pattern(pattern.into()))
            }
            e => Self::from_expression_simple(e, strict),
        }
    }

    /// Converts the left-hand-side Expression of an assignment expression into an [`AssignTarget`].
    /// Returns `None` if the given Expression is an invalid left-hand-side for a assignment expression.
    ///
    /// The `AssignmentTargetType` of the expression must be `simple`.
    #[must_use]
    pub fn from_expression_simple(expression: &Expression<'arena>, strict: bool) -> Option<Self> {
        match expression {
            Expression::Identifier(id)
                if strict && (id.sym() == Sym::EVAL || id.sym() == Sym::ARGUMENTS) =>
            {
                None
            }
            Expression::Identifier(id) => Some(Self::Identifier(*id)),
            Expression::PropertyAccess(access) => Some(Self::Access(access.clone())),
            Expression::Parenthesized(p) => Self::from_expression_simple(p.expression(), strict),
            _ => None,
        }
    }
}

impl<'arena> Spanned for AssignTarget<'arena> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            AssignTarget::Identifier(identifier) => identifier.span(),
            AssignTarget::Access(property_access) => property_access.span(),
            AssignTarget::Pattern(pattern) => pattern.span(),
        }
    }
}

impl<'arena> ToInternedString for AssignTarget<'arena> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Identifier(id) => id.to_interned_string(interner),
            Self::Access(access) => access.to_interned_string(interner),
            Self::Pattern(pattern) => pattern.to_interned_string(interner),
        }
    }
}

impl<'arena> From<Identifier> for AssignTarget<'arena> {
    #[inline]
    fn from(target: Identifier) -> Self {
        Self::Identifier(target)
    }
}

impl<'arena> VisitWith<'arena> for AssignTarget<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        match self {
            Self::Identifier(id) => visitor.visit_identifier(id),
            Self::Access(pa) => visitor.visit_property_access(pa),
            Self::Pattern(pat) => visitor.visit_pattern(pat),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        match self {
            Self::Identifier(id) => visitor.visit_identifier_mut(id),
            Self::Access(pa) => visitor.visit_property_access_mut(pa),
            Self::Pattern(pat) => visitor.visit_pattern_mut(pat),
        }
    }
}
