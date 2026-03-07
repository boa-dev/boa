//! Property access expressions, as defined by the [spec].
//!
//! [Property access expressions][access] provide two ways to access properties of an object: *dot notation*
//! and *bracket notation*.
//! - *Dot notation* is mostly used when the name of the property is static, and a valid Javascript
//!   identifier e.g. `obj.prop`, `arr.$val`.
//! - *Bracket notation* is used when the name of the property is either variable, not a valid
//!   identifier or a symbol e.g. `arr[var]`, `arr[5]`, `arr[Symbol.iterator]`.
//!
//! A property access expression can be represented by a [`SimplePropertyAccess`] (`x.y`), a
//! [`PrivatePropertyAccess`] (`x.#y`) or a [`SuperPropertyAccess`] (`super["y"]`), each of them with
//! slightly different semantics overall.
//!
//! [spec]: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-property-accessors
//! [access]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_Accessors

use crate::expression::Expression;
use crate::function::PrivateName;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{Span, Spanned};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

use super::Identifier;

/// A property access field.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyAccessField<'arena> {
    /// A constant property field, such as `x.prop`.
    Const(Identifier),
    /// An expression property field, such as `x["val"]`.
    Expr(Box<Expression<'arena>>),
}

impl Spanned for PropertyAccessField<'_> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Const(identifier) => identifier.span(),
            Self::Expr(expression) => expression.span(),
        }
    }
}

impl From<Identifier> for PropertyAccessField<'_> {
    #[inline]
    fn from(id: Identifier) -> Self {
        Self::Const(id)
    }
}

impl<'arena> From<Expression<'arena>> for PropertyAccessField<'arena> {
    #[inline]
    fn from(expr: Expression<'arena>) -> Self {
        Self::Expr(Box::new(expr))
    }
}

impl<'arena> VisitWith<'arena> for PropertyAccessField<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        match self {
            Self::Const(sym) => visitor.visit_sym(sym.sym_ref()),
            Self::Expr(expr) => visitor.visit_expression(expr),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        match self {
            Self::Const(sym) => visitor.visit_sym_mut(sym.sym_mut()),
            Self::Expr(expr) => visitor.visit_expression_mut(&mut *expr),
        }
    }
}

/// A property access expression.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyAccess<'arena> {
    /// A simple property access (`x.prop`).
    Simple(SimplePropertyAccess<'arena>),
    /// A property access of a private property (`x.#priv`).
    Private(PrivatePropertyAccess<'arena>),
    /// A property access of a `super` reference. (`super["prop"]`).
    Super(SuperPropertyAccess<'arena>),
}

impl Spanned for PropertyAccess<'_> {
    #[inline]
    fn span(&self) -> Span {
        match self {
            Self::Simple(access) => access.span(),
            Self::Private(access) => access.span(),
            Self::Super(access) => access.span(),
        }
    }
}

impl ToInternedString for PropertyAccess<'_> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Simple(s) => s.to_interned_string(interner),
            Self::Private(p) => p.to_interned_string(interner),
            Self::Super(s) => s.to_interned_string(interner),
        }
    }
}

impl<'arena> From<PropertyAccess<'arena>> for Expression<'arena> {
    #[inline]
    fn from(access: PropertyAccess<'arena>) -> Self {
        Self::PropertyAccess(access)
    }
}

impl<'arena> VisitWith<'arena> for PropertyAccess<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        match self {
            Self::Simple(spa) => visitor.visit_simple_property_access(spa),
            Self::Private(ppa) => visitor.visit_private_property_access(ppa),
            Self::Super(supa) => visitor.visit_super_property_access(supa),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        match self {
            Self::Simple(spa) => visitor.visit_simple_property_access_mut(spa),
            Self::Private(ppa) => visitor.visit_private_property_access_mut(ppa),
            Self::Super(supa) => visitor.visit_super_property_access_mut(supa),
        }
    }
}

/// A simple property access, where the target object is an [`Expression`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct SimplePropertyAccess<'arena> {
    target: Box<Expression<'arena>>,
    field: PropertyAccessField<'arena>,
}

impl<'arena> SimplePropertyAccess<'arena> {
    /// Gets the target object of the property access.
    #[inline]
    #[must_use]
    pub const fn target(&self) -> &Expression<'arena> {
        &self.target
    }

    /// Gets the accessed field of the target object.
    #[inline]
    #[must_use]
    pub const fn field(&self) -> &PropertyAccessField<'arena> {
        &self.field
    }

    /// Creates a `PropertyAccess` AST Expression.
    pub fn new<F>(target: Expression<'arena>, field: F) -> Self
    where
        F: Into<PropertyAccessField<'arena>>,
    {
        Self {
            target: target.into(),
            field: field.into(),
        }
    }
}

impl Spanned for SimplePropertyAccess<'_> {
    #[inline]
    fn span(&self) -> Span {
        Span::new(self.target.span().start(), self.field.span().end())
    }
}

impl ToInternedString for SimplePropertyAccess<'_> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let target = self.target.to_interned_string(interner);
        match self.field {
            PropertyAccessField::Const(ident) => {
                format!("{target}.{}", interner.resolve_expect(ident.sym()))
            }
            PropertyAccessField::Expr(ref expr) => {
                format!("{target}[{}]", expr.to_interned_string(interner))
            }
        }
    }
}

impl<'arena> From<SimplePropertyAccess<'arena>> for PropertyAccess<'arena> {
    #[inline]
    fn from(access: SimplePropertyAccess<'arena>) -> Self {
        Self::Simple(access)
    }
}

impl<'arena> VisitWith<'arena> for SimplePropertyAccess<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_expression(&self.target)?;
        visitor.visit_property_access_field(&self.field)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_expression_mut(&mut self.target)?;
        visitor.visit_property_access_field_mut(&mut self.field)
    }
}

/// An access expression to a class object's [private fields][mdn].
///
/// Private property accesses differ slightly from plain property accesses, since the accessed
/// property must be prefixed by `#`, and the bracket notation is not allowed. For example,
/// `this.#a` is a valid private property access.
///
/// This expression corresponds to the [`MemberExpression.PrivateIdentifier`][spec] production.
///
/// [spec]: https://tc39.es/ecma262/#prod-MemberExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_class_fields
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct PrivatePropertyAccess<'arena> {
    target: Box<Expression<'arena>>,
    field: PrivateName,
    span: Span,
}

impl<'arena> PrivatePropertyAccess<'arena> {
    /// Creates a `GetPrivateField` AST Expression.
    #[inline]
    #[must_use]
    pub fn new(value: Expression<'arena>, field: PrivateName, span: Span) -> Self {
        Self {
            target: value.into(),
            field,
            span,
        }
    }

    /// Gets the original object from where to get the field from.
    #[inline]
    #[must_use]
    pub const fn target(&self) -> &Expression<'arena> {
        &self.target
    }

    /// Gets the name of the field to retrieve.
    #[inline]
    #[must_use]
    pub const fn field(&self) -> PrivateName {
        self.field
    }
}

impl Spanned for PrivatePropertyAccess<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl ToInternedString for PrivatePropertyAccess<'_> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}.#{}",
            self.target.to_interned_string(interner),
            interner.resolve_expect(self.field.description())
        )
    }
}

impl<'arena> From<PrivatePropertyAccess<'arena>> for PropertyAccess<'arena> {
    #[inline]
    fn from(access: PrivatePropertyAccess<'arena>) -> Self {
        Self::Private(access)
    }
}

impl<'arena> VisitWith<'arena> for PrivatePropertyAccess<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_expression(&self.target)?;
        visitor.visit_private_name(&self.field)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_expression_mut(&mut self.target)?;
        visitor.visit_private_name_mut(&mut self.field)
    }
}

/// A property access of an object's parent, as defined by the [spec].
///
/// A `SuperPropertyAccess` is much like a regular [`PropertyAccess`], but where its `target` object
/// is not a regular object, but a reference to the parent object of the current object ([`super`][mdn]).
///
/// [spec]: https://tc39.es/ecma262/#prod-SuperProperty
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/super
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct SuperPropertyAccess<'arena> {
    field: PropertyAccessField<'arena>,
    span: Span,
}

impl<'arena> SuperPropertyAccess<'arena> {
    /// Creates a new property access field node.
    #[must_use]
    pub const fn new(field: PropertyAccessField<'arena>, span: Span) -> Self {
        Self { field, span }
    }

    /// Gets the name of the field to retrieve.
    #[inline]
    #[must_use]
    pub const fn field(&self) -> &PropertyAccessField<'arena> {
        &self.field
    }
}

impl Spanned for SuperPropertyAccess<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl ToInternedString for SuperPropertyAccess<'_> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match &self.field {
            PropertyAccessField::Const(field) => {
                format!("super.{}", interner.resolve_expect(field.sym()))
            }
            PropertyAccessField::Expr(field) => {
                format!("super[{}]", field.to_interned_string(interner))
            }
        }
    }
}

impl<'arena> From<SuperPropertyAccess<'arena>> for PropertyAccess<'arena> {
    #[inline]
    fn from(access: SuperPropertyAccess<'arena>) -> Self {
        Self::Super(access)
    }
}

impl<'arena> VisitWith<'arena> for SuperPropertyAccess<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_property_access_field(&self.field)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_property_access_field_mut(&mut self.field)
    }
}
