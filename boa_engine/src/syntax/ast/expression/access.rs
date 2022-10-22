use crate::syntax::ast::{expression::Expression, ContainsSymbol};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyAccessField {
    Const(Sym),
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

/// This property accessor provides access to an object's properties by using the
/// [bracket notation][mdn].
///
/// In the `object[property_name]` syntax, the `property_name` is just a string or
/// [Symbol][symbol]. So, it can be any string, including '1foo', '!bar!', or even ' ' (a
/// space).
///
/// One can think of an object as an associative array (a.k.a. map, dictionary, hash, lookup
/// table). The keys in this array are the names of the object's properties.
///
/// It's typical when speaking of an object's properties to make a distinction between
/// properties and methods. However, the property/method distinction is little more than a
/// convention. A method is simply a property that can be called (for example, if it has a
/// reference to a Function instance as its value).
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-property-accessors
/// [symbol]: https://developer.mozilla.org/en-US/docs/Glossary/Symbol
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_accessors#Bracket_notation
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyAccess {
    target: Box<Expression>,
    field: PropertyAccessField,
}

impl PropertyAccess {
    #[inline]
    pub fn target(&self) -> &Expression {
        &self.target
    }

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

impl ToInternedString for PropertyAccess {
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

impl From<PropertyAccess> for Expression {
    #[inline]
    fn from(access: PropertyAccess) -> Self {
        Self::PropertyAccess(access)
    }
}

/// This property accessor provides access to an class object's private fields.
///
/// This expression can be described as ` MemberExpression.PrivateIdentifier`
/// Example: `this.#a`
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
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

impl From<PrivatePropertyAccess> for Expression {
    #[inline]
    fn from(access: PrivatePropertyAccess) -> Self {
        Self::PrivatePropertyAccess(access)
    }
}

/// The `super` keyword is used to access fields on an object's parent.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
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
        self.field.contains(symbol)
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

impl From<SuperPropertyAccess> for Expression {
    #[inline]
    fn from(access: SuperPropertyAccess) -> Self {
        Self::SuperPropertyAccess(access)
    }
}
