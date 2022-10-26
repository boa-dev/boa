//! Property access expressions, as defined by the [spec].
//!
//! [Property access expressions][access] provide two ways to access properties of an object: *dot notation*
//! and *bracket notation*.
//! - *Dot notation* is mostly used when the name of the property is static, and a valid Javascript
//! identifier e.g. `obj.prop`, `arr.$val`.
//! - *Bracket notation* is used when the name of the property is either variable, not a valid
//! identifier or a symbol e.g. `arr[var]`, `arr[5]`, `arr[Symbol.iterator]`.
//!
//! A property access expression can be represented by a [`SimplePropertyAccess`] (`x.y`), a
//! [`PrivatePropertyAccess`] (`x.#y`) or a [`SuperPropertyAccess`] (`super["y"]`), each of them with
//! slightly different semantics overall.
//!
//! [spec]: https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-property-accessors
//! [access]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_Accessors

use crate::syntax::ast::{expression::Expression, ContainsSymbol};
use boa_interner::{Interner, Sym, ToInternedString};

/// A property access field.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyAccessField {
    /// A constant property field, such as `x.prop`.
    Const(Sym),
    /// An expression property field, such as `x["val"]`.
    Expr(Box<Expression>),
}

impl PropertyAccessField {
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            PropertyAccessField::Const(_) => false,
            PropertyAccessField::Expr(expr) => expr.contains_arguments(),
        }
    }
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            PropertyAccessField::Const(_) => false,
            PropertyAccessField::Expr(expr) => expr.contains(symbol),
        }
    }
}

impl From<Sym> for PropertyAccessField {
    #[inline]
    fn from(id: Sym) -> Self {
        Self::Const(id)
    }
}

impl From<Expression> for PropertyAccessField {
    #[inline]
    fn from(expr: Expression) -> Self {
        Self::Expr(Box::new(expr))
    }
}

/// A property access expression.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyAccess {
    /// A simple property access (`x.prop`).
    Simple(SimplePropertyAccess),
    /// A property access of a private property (`x.#priv`).
    Private(PrivatePropertyAccess),
    /// A property access of a `super` reference. (`super["prop"]`).
    Super(SuperPropertyAccess),
}

impl PropertyAccess {
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            PropertyAccess::Simple(s) => s.contains_arguments(),
            PropertyAccess::Private(p) => p.contains_arguments(),
            PropertyAccess::Super(s) => s.contains_arguments(),
        }
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            PropertyAccess::Simple(s) => s.contains(symbol),
            PropertyAccess::Private(p) => p.contains(symbol),
            PropertyAccess::Super(s) => s.contains(symbol),
        }
    }
}

impl ToInternedString for PropertyAccess {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            PropertyAccess::Simple(s) => s.to_interned_string(interner),
            PropertyAccess::Private(p) => p.to_interned_string(interner),
            PropertyAccess::Super(s) => s.to_interned_string(interner),
        }
    }
}

impl From<PropertyAccess> for Expression {
    #[inline]
    fn from(access: PropertyAccess) -> Self {
        Self::PropertyAccess(access)
    }
}

/// A simple property access, where the target object is an [`Expression`].
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SimplePropertyAccess {
    target: Box<Expression>,
    field: PropertyAccessField,
}

impl SimplePropertyAccess {
    /// Gets the target object of the property access.
    #[inline]
    pub fn target(&self) -> &Expression {
        &self.target
    }

    /// Gets the accessed field of the target object.
    #[inline]
    pub fn field(&self) -> &PropertyAccessField {
        &self.field
    }

    /// Creates a `PropertyAccess` AST Expression.
    #[inline]
    pub fn new<F>(target: Expression, field: F) -> Self
    where
        F: Into<PropertyAccessField>,
    {
        Self {
            target: target.into(),
            field: field.into(),
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.target.contains_arguments() || self.field.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.target.contains(symbol) || self.field.contains(symbol)
    }
}

impl ToInternedString for SimplePropertyAccess {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let target = self.target.to_interned_string(interner);
        match self.field {
            PropertyAccessField::Const(sym) => format!("{target}.{}", interner.resolve_expect(sym)),
            PropertyAccessField::Expr(ref expr) => {
                format!("{target}[{}]", expr.to_interned_string(interner))
            }
        }
    }
}

impl From<SimplePropertyAccess> for PropertyAccess {
    #[inline]
    fn from(access: SimplePropertyAccess) -> Self {
        Self::Simple(access)
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PrivatePropertyAccess {
    target: Box<Expression>,
    field: Sym,
}

impl PrivatePropertyAccess {
    /// Creates a `GetPrivateField` AST Expression.
    #[inline]
    pub fn new(value: Expression, field: Sym) -> Self {
        Self {
            target: value.into(),
            field,
        }
    }

    /// Gets the original object from where to get the field from.
    #[inline]
    pub fn target(&self) -> &Expression {
        &self.target
    }

    /// Gets the name of the field to retrieve.
    #[inline]
    pub fn field(&self) -> Sym {
        self.field
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.target.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.target.contains(symbol)
    }
}

impl ToInternedString for PrivatePropertyAccess {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{}.#{}",
            self.target.to_interned_string(interner),
            interner.resolve_expect(self.field)
        )
    }
}

impl From<PrivatePropertyAccess> for PropertyAccess {
    #[inline]
    fn from(access: PrivatePropertyAccess) -> Self {
        Self::Private(access)
    }
}

/// A property access of an object's parent, as defined by the [spec].
///
/// A `SuperPropertyAccess` is much like a regular [`PropertyAccess`], but where its `target` object
/// is not a regular object, but a reference to the parent object of the current object ([`super`][mdn]).
///
/// [spec]: https://tc39.es/ecma262/#prod-SuperProperty
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/super
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SuperPropertyAccess {
    field: PropertyAccessField,
}

impl SuperPropertyAccess {
    pub(in crate::syntax) fn new(field: PropertyAccessField) -> Self {
        Self { field }
    }

    /// Gets the name of the field to retrieve.
    #[inline]
    pub fn field(&self) -> &PropertyAccessField {
        &self.field
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.field.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        symbol == ContainsSymbol::SuperProperty || self.field.contains(symbol)
    }
}

impl ToInternedString for SuperPropertyAccess {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        match &self.field {
            PropertyAccessField::Const(field) => {
                format!("super.{}", interner.resolve_expect(*field))
            }
            PropertyAccessField::Expr(field) => {
                format!("super[{}]", field.to_interned_string(interner))
            }
        }
    }
}

impl From<SuperPropertyAccess> for PropertyAccess {
    #[inline]
    fn from(access: SuperPropertyAccess) -> Self {
        Self::Super(access)
    }
}
