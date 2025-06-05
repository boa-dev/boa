//! Property definition related types, used in object literals and class definitions.

use super::Expression;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use boa_interner::{Interner, Sym, ToInternedString};
use core::ops::ControlFlow;

/// `PropertyName` can be either a literal or computed.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-PropertyName
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyName {
    /// A `Literal` property name can be either an identifier, a string or a numeric literal.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-LiteralPropertyName
    Literal(Sym),

    /// A `Computed` property name is an expression that gets evaluated and converted into a property name.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-ComputedPropertyName
    Computed(Expression),
}

impl PropertyName {
    /// Returns the literal property name if it exists.
    #[must_use]
    pub const fn literal(&self) -> Option<Sym> {
        if let Self::Literal(sym) = self {
            Some(*sym)
        } else {
            None
        }
    }

    /// Returns the expression if the property name is computed.
    #[must_use]
    pub const fn computed(&self) -> Option<&Expression> {
        if let Self::Computed(expr) = self {
            Some(expr)
        } else {
            None
        }
    }

    /// Returns either the literal property name or the computed const string property name.
    #[must_use]
    pub const fn prop_name(&self) -> Option<Sym> {
        match self {
            Self::Literal(sym) => Some(*sym),
            Self::Computed(Expression::Literal(lit)) => lit.as_string(),
            Self::Computed(_) => None,
        }
    }
}

impl ToInternedString for PropertyName {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Literal(key) => interner.resolve_expect(*key).to_string(),
            Self::Computed(key) => format!("[{}]", key.to_interned_string(interner)),
        }
    }
}

impl From<Sym> for PropertyName {
    fn from(name: Sym) -> Self {
        Self::Literal(name)
    }
}

impl From<Expression> for PropertyName {
    fn from(name: Expression) -> Self {
        Self::Computed(name)
    }
}

impl VisitWith for PropertyName {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::Literal(sym) => visitor.visit_sym(sym),
            Self::Computed(expr) => visitor.visit_expression(expr),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::Literal(sym) => visitor.visit_sym_mut(sym),
            Self::Computed(expr) => visitor.visit_expression_mut(expr),
        }
    }
}

/// The kind of a method definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MethodDefinitionKind {
    /// A getter method.
    Get,

    /// A setter method.
    Set,

    /// An ordinary method.
    Ordinary,

    /// A generator method.
    Generator,

    /// An async generator method.
    AsyncGenerator,

    /// An async method.
    Async,
}
