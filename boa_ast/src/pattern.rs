//! A pattern binding or assignment node.
//!
//! A [`Pattern`] Corresponds to the [`BindingPattern`][spec1] and the [`AssignmentPattern`][spec2]
//! nodes, each of which is used in different situations and have slightly different grammars.
//! For example, a variable declaration combined with a destructuring expression is a `BindingPattern`:
//!
//! ```Javascript
//! const obj = { a: 1, b: 2 };
//! const { a, b } = obj; // BindingPattern
//! ```
//!
//! On the other hand, a simple destructuring expression with already declared variables is called
//! an `AssignmentPattern`:
//!
//! ```Javascript
//! let a = 1;
//! let b = 3;
//! [a, b] = [b, a]; // AssignmentPattern
//! ```
//!
//! [spec1]: https://tc39.es/ecma262/#prod-BindingPattern
//! [spec2]: https://tc39.es/ecma262/#prod-AssignmentPattern
//! [destr]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Destructuring_assignment

use crate::{
    expression::{access::PropertyAccess, Identifier},
    property::PropertyName,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    Expression,
};
use boa_interner::{Interner, ToInternedString};
use core::ops::ControlFlow;

/// An object or array pattern binding or assignment.
///
/// See the [module level documentation][self] for more information.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    /// An object pattern (`let {a, b, c} = object`).
    Object(ObjectPattern),
    /// An array pattern (`[a, b, c] = array`).
    Array(ArrayPattern),
}

impl From<ObjectPattern> for Pattern {
    fn from(obj: ObjectPattern) -> Self {
        Pattern::Object(obj)
    }
}

impl From<ArrayPattern> for Pattern {
    fn from(obj: ArrayPattern) -> Self {
        Pattern::Array(obj)
    }
}

impl From<Vec<ObjectPatternElement>> for Pattern {
    fn from(elements: Vec<ObjectPatternElement>) -> Self {
        ObjectPattern::new(elements.into()).into()
    }
}
impl From<Vec<ArrayPatternElement>> for Pattern {
    fn from(elements: Vec<ArrayPatternElement>) -> Self {
        ArrayPattern::new(elements.into()).into()
    }
}

impl ToInternedString for Pattern {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match &self {
            Pattern::Object(o) => o.to_interned_string(interner),
            Pattern::Array(a) => a.to_interned_string(interner),
        }
    }
}

impl VisitWith for Pattern {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Pattern::Object(op) => visitor.visit_object_pattern(op),
            Pattern::Array(ap) => visitor.visit_array_pattern(ap),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Pattern::Object(op) => visitor.visit_object_pattern_mut(op),
            Pattern::Array(ap) => visitor.visit_array_pattern_mut(ap),
        }
    }
}

/// An object binding or assignment pattern.
///
/// Corresponds to the [`ObjectBindingPattern`][spec1] and the [`ObjectAssignmentPattern`][spec2]
/// Parse Nodes.
///
/// For more information on what is a valid binding in an object pattern, see [`ObjectPatternElement`].
///
/// [spec1]: https://tc39.es/ecma262/#prod-ObjectBindingPattern
/// [spec2]: https://tc39.es/ecma262/#prod-ObjectAssignmentPattern
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectPattern(Box<[ObjectPatternElement]>);

impl From<Vec<ObjectPatternElement>> for ObjectPattern {
    fn from(elements: Vec<ObjectPatternElement>) -> Self {
        Self(elements.into())
    }
}

impl ToInternedString for ObjectPattern {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "{".to_owned();
        for (i, binding) in self.0.iter().enumerate() {
            let binding = binding.to_interned_string(interner);
            let str = if i == self.0.len() - 1 {
                format!("{binding} ")
            } else {
                format!("{binding},")
            };

            buf.push_str(&str);
        }
        if self.0.is_empty() {
            buf.push(' ');
        }
        buf.push('}');
        buf
    }
}

impl ObjectPattern {
    /// Creates a new object binding pattern.
    #[inline]
    #[must_use]
    pub fn new(bindings: Box<[ObjectPatternElement]>) -> Self {
        Self(bindings)
    }

    /// Gets the bindings for the object binding pattern.
    #[inline]
    #[must_use]
    pub fn bindings(&self) -> &[ObjectPatternElement] {
        &self.0
    }

    /// Returns true if the object binding pattern has a rest element.
    #[inline]
    #[must_use]
    pub fn has_rest(&self) -> bool {
        matches!(
            self.0.last(),
            Some(ObjectPatternElement::RestProperty { .. })
        )
    }
}

impl VisitWith for ObjectPattern {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for elem in self.0.iter() {
            try_break!(visitor.visit_object_pattern_element(elem));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for elem in self.0.iter_mut() {
            try_break!(visitor.visit_object_pattern_element_mut(elem));
        }
        ControlFlow::Continue(())
    }
}

/// An array binding or assignment pattern.
///
/// Corresponds to the [`ArrayBindingPattern`][spec1] and the [`ArrayAssignmentPattern`][spec2]
/// Parse Nodes.
///
/// For more information on what is a valid binding in an array pattern, see [`ArrayPatternElement`].
///
/// [spec1]: https://tc39.es/ecma262/#prod-ArrayBindingPattern
/// [spec2]: https://tc39.es/ecma262/#prod-ArrayAssignmentPattern
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayPattern(Box<[ArrayPatternElement]>);

impl From<Vec<ArrayPatternElement>> for ArrayPattern {
    fn from(elements: Vec<ArrayPatternElement>) -> Self {
        Self(elements.into())
    }
}

impl ToInternedString for ArrayPattern {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "[".to_owned();
        for (i, binding) in self.0.iter().enumerate() {
            if i == self.0.len() - 1 {
                match binding {
                    ArrayPatternElement::Elision => {
                        buf.push_str(&format!("{}, ", binding.to_interned_string(interner)));
                    }
                    _ => buf.push_str(&format!("{} ", binding.to_interned_string(interner))),
                }
            } else {
                buf.push_str(&format!("{},", binding.to_interned_string(interner)));
            }
        }
        buf.push(']');
        buf
    }
}

impl ArrayPattern {
    /// Creates a new array binding pattern.
    #[inline]
    #[must_use]
    pub fn new(bindings: Box<[ArrayPatternElement]>) -> Self {
        Self(bindings)
    }

    /// Gets the bindings for the array binding pattern.
    #[inline]
    #[must_use]
    pub fn bindings(&self) -> &[ArrayPatternElement] {
        &self.0
    }
}

impl VisitWith for ArrayPattern {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for elem in self.0.iter() {
            try_break!(visitor.visit_array_pattern_element(elem));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for elem in self.0.iter_mut() {
            try_break!(visitor.visit_array_pattern_element_mut(elem));
        }
        ControlFlow::Continue(())
    }
}

/// The different types of bindings that an [`ObjectPattern`] may contain.
///
/// Corresponds to the [`BindingProperty`][spec1] and the [`AssignmentProperty`][spec2] nodes.
///
/// [spec1]: https://tc39.es/ecma262/#prod-BindingProperty
/// [spec2]: https://tc39.es/ecma262/#prod-AssignmentProperty
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum ObjectPatternElement {
    /// SingleName represents one of the following properties:
    ///
    /// - `SingleName` with an identifier and an optional default initializer.
    /// - `BindingProperty` with an property name and a `SingleNameBinding` as  the `BindingElement`.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - SingleNameBinding][spec1]
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingProperty][spec2]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-SingleNameBinding
    /// [spec2]: https://tc39.es/ecma262/#prod-BindingProperty
    SingleName {
        /// The identifier name of the property to be destructured.
        name: PropertyName,
        /// The variable name where the property value will be stored.
        ident: Identifier,
        /// An optional default value for the variable, in case the property doesn't exist.
        default_init: Option<Expression>,
    },

    /// RestProperty represents a `BindingRestProperty` with an identifier.
    ///
    /// It also includes a list of the property keys that should be excluded from the rest,
    /// because they where already assigned.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingRestProperty][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingRestProperty
    RestProperty {
        /// The variable name where the unassigned properties will be stored.
        ident: Identifier,
        /// A list of the excluded property keys that were already destructured.
        excluded_keys: Vec<Identifier>,
    },

    /// AssignmentGetField represents an AssignmentProperty with an expression field member expression AssignmentElement.
    ///
    /// Note: According to the spec this is not part of an ObjectBindingPattern.
    /// This is only used when a object literal is used to cover an AssignmentPattern.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentProperty
    AssignmentPropertyAccess {
        /// The identifier name of the property to be destructured.
        name: PropertyName,
        /// The property access where the property value will be destructured.
        access: PropertyAccess,
        /// An optional default value for the variable, in case the property doesn't exist.
        default_init: Option<Expression>,
    },

    /// AssignmentRestProperty represents a rest property with a DestructuringAssignmentTarget.
    ///
    /// Note: According to the spec this is not part of an ObjectBindingPattern.
    /// This is only used when a object literal is used to cover an AssignmentPattern.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentRestProperty
    AssignmentRestPropertyAccess {
        /// The property access where the unassigned properties will be stored.
        access: PropertyAccess,
        /// A list of the excluded property keys that were already destructured.
        excluded_keys: Vec<Identifier>,
    },

    /// Pattern represents a property with a `Pattern` as the element.
    ///
    /// Additionally to the identifier of the new property and the nested pattern,
    /// this may also include an optional default initializer.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingProperty][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingProperty
    Pattern {
        /// The identifier name of the property to be destructured.
        name: PropertyName,
        /// The pattern where the property value will be destructured.
        pattern: Pattern,
        /// An optional default value for the variable, in case the property doesn't exist.
        default_init: Option<Expression>,
    },
}

impl ToInternedString for ObjectPatternElement {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::SingleName {
                ident,
                name,
                default_init,
            } => {
                let mut buf = match name {
                    PropertyName::Literal(name) if name == ident => {
                        format!(" {}", interner.resolve_expect(ident.sym()))
                    }
                    PropertyName::Literal(name) => {
                        format!(
                            " {} : {}",
                            interner.resolve_expect(*name),
                            interner.resolve_expect(ident.sym())
                        )
                    }
                    PropertyName::Computed(node) => {
                        format!(
                            " [{}] : {}",
                            node.to_interned_string(interner),
                            interner.resolve_expect(ident.sym())
                        )
                    }
                };
                if let Some(ref init) = default_init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
            Self::RestProperty {
                ident,
                excluded_keys: _,
            } => {
                format!(" ... {}", interner.resolve_expect(ident.sym()))
            }
            Self::AssignmentRestPropertyAccess { access, .. } => {
                format!(" ... {}", access.to_interned_string(interner))
            }
            Self::AssignmentPropertyAccess {
                name,
                access,
                default_init,
            } => {
                let mut buf = match name {
                    PropertyName::Literal(name) => {
                        format!(
                            " {} : {}",
                            interner.resolve_expect(*name),
                            access.to_interned_string(interner)
                        )
                    }
                    PropertyName::Computed(node) => {
                        format!(
                            " [{}] : {}",
                            node.to_interned_string(interner),
                            access.to_interned_string(interner)
                        )
                    }
                };
                if let Some(init) = &default_init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
            Self::Pattern {
                name,
                pattern,
                default_init,
            } => {
                let mut buf = match name {
                    PropertyName::Literal(name) => {
                        format!(
                            " {} : {}",
                            interner.resolve_expect(*name),
                            pattern.to_interned_string(interner),
                        )
                    }
                    PropertyName::Computed(node) => {
                        format!(
                            " [{}] : {}",
                            node.to_interned_string(interner),
                            pattern.to_interned_string(interner),
                        )
                    }
                };
                if let Some(ref init) = default_init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
        }
    }
}

impl VisitWith for ObjectPatternElement {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            ObjectPatternElement::SingleName {
                name,
                ident,
                default_init,
            } => {
                try_break!(visitor.visit_property_name(name));
                try_break!(visitor.visit_identifier(ident));
                if let Some(expr) = default_init {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ObjectPatternElement::RestProperty { ident, .. } => visitor.visit_identifier(ident),
            ObjectPatternElement::AssignmentPropertyAccess {
                name,
                access,
                default_init,
            } => {
                try_break!(visitor.visit_property_name(name));
                try_break!(visitor.visit_property_access(access));
                if let Some(expr) = default_init {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ObjectPatternElement::AssignmentRestPropertyAccess { access, .. } => {
                visitor.visit_property_access(access)
            }
            ObjectPatternElement::Pattern {
                name,
                pattern,
                default_init,
            } => {
                try_break!(visitor.visit_property_name(name));
                try_break!(visitor.visit_pattern(pattern));
                if let Some(expr) = default_init {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            ObjectPatternElement::SingleName {
                name,
                ident,
                default_init,
            } => {
                try_break!(visitor.visit_property_name_mut(name));
                try_break!(visitor.visit_identifier_mut(ident));
                if let Some(expr) = default_init {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ObjectPatternElement::RestProperty { ident, .. } => visitor.visit_identifier_mut(ident),
            ObjectPatternElement::AssignmentPropertyAccess {
                name,
                access,
                default_init,
            } => {
                try_break!(visitor.visit_property_name_mut(name));
                try_break!(visitor.visit_property_access_mut(access));
                if let Some(expr) = default_init {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ObjectPatternElement::AssignmentRestPropertyAccess { access, .. } => {
                visitor.visit_property_access_mut(access)
            }
            ObjectPatternElement::Pattern {
                name,
                pattern,
                default_init,
            } => {
                try_break!(visitor.visit_property_name_mut(name));
                try_break!(visitor.visit_pattern_mut(pattern));
                if let Some(expr) = default_init {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
        }
    }
}

/// The different types of bindings that an array binding pattern may contain.
///
/// Corresponds to the [`BindingElement`][spec1] and the [`AssignmentElement`][spec2] nodes.
///
/// [spec1]: https://tc39.es/ecma262/#prod-BindingElement
/// [spec2]: https://tc39.es/ecma262/#prod-AssignmentElement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum ArrayPatternElement {
    /// Elision represents the elision of an item in the array binding pattern.
    ///
    /// An `Elision` may occur at multiple points in the pattern and may be multiple elisions.
    /// This variant strictly represents one elision. If there are multiple, this should be used multiple times.
    ///
    /// More information:
    ///  - [ECMAScript reference: 13.2.4 Array Initializer - Elision][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-Elision
    Elision,

    /// SingleName represents a `SingleName` with an identifier and an optional default initializer.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - SingleNameBinding][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-SingleNameBinding
    SingleName {
        /// The variable name where the index element will be stored.
        ident: Identifier,
        /// An optional default value for the variable, in case the index element doesn't exist.
        default_init: Option<Expression>,
    },

    /// PropertyAccess represents a binding with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ArrayBindingPattern.
    /// This is only used when a array literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    PropertyAccess {
        /// The property access where the index element will be stored.
        access: PropertyAccess,
    },

    /// Pattern represents a `Pattern` in an `Element` of an array pattern.
    ///
    /// The pattern and the optional default initializer are both stored in the DeclarationPattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingElement
    Pattern {
        /// The pattern where the index element will be stored.
        pattern: Pattern,
        /// An optional default value for the pattern, in case the index element doesn't exist.
        default_init: Option<Expression>,
    },

    /// SingleNameRest represents a `BindingIdentifier` in a `BindingRestElement` of an array pattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingRestElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingRestElement
    SingleNameRest {
        /// The variable where the unassigned index elements will be stored.
        ident: Identifier,
    },

    /// PropertyAccess represents a rest (spread operator) with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ArrayBindingPattern.
    /// This is only used when a array literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    PropertyAccessRest {
        /// The property access where the unassigned index elements will be stored.
        access: PropertyAccess,
    },

    /// PatternRest represents a `Pattern` in a `RestElement` of an array pattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingRestElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingRestElement
    PatternRest {
        /// The pattern where the unassigned index elements will be stored.
        pattern: Pattern,
    },
}

impl ToInternedString for ArrayPatternElement {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Elision => " ".to_owned(),
            Self::SingleName {
                ident,
                default_init,
            } => {
                let mut buf = format!(" {}", interner.resolve_expect(ident.sym()));
                if let Some(ref init) = default_init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
            Self::PropertyAccess { access } => {
                format!(" {}", access.to_interned_string(interner))
            }
            Self::Pattern {
                pattern,
                default_init,
            } => {
                let mut buf = format!(" {}", pattern.to_interned_string(interner));
                if let Some(init) = default_init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
            Self::SingleNameRest { ident } => {
                format!(" ... {}", interner.resolve_expect(ident.sym()))
            }
            Self::PropertyAccessRest { access } => {
                format!(" ... {}", access.to_interned_string(interner))
            }
            Self::PatternRest { pattern } => {
                format!(" ... {}", pattern.to_interned_string(interner))
            }
        }
    }
}

impl VisitWith for ArrayPatternElement {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            ArrayPatternElement::SingleName {
                ident,
                default_init,
            } => {
                try_break!(visitor.visit_identifier(ident));
                if let Some(expr) = default_init {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ArrayPatternElement::PropertyAccess { access }
            | ArrayPatternElement::PropertyAccessRest { access } => {
                visitor.visit_property_access(access)
            }
            ArrayPatternElement::Pattern {
                pattern,
                default_init,
            } => {
                try_break!(visitor.visit_pattern(pattern));
                if let Some(expr) = default_init {
                    visitor.visit_expression(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ArrayPatternElement::SingleNameRest { ident } => visitor.visit_identifier(ident),
            ArrayPatternElement::PatternRest { pattern } => visitor.visit_pattern(pattern),
            ArrayPatternElement::Elision => {
                // special case to be handled by user
                ControlFlow::Continue(())
            }
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            ArrayPatternElement::SingleName {
                ident,
                default_init,
            } => {
                try_break!(visitor.visit_identifier_mut(ident));
                if let Some(expr) = default_init {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ArrayPatternElement::PropertyAccess { access }
            | ArrayPatternElement::PropertyAccessRest { access } => {
                visitor.visit_property_access_mut(access)
            }
            ArrayPatternElement::Pattern {
                pattern,
                default_init,
            } => {
                try_break!(visitor.visit_pattern_mut(pattern));
                if let Some(expr) = default_init {
                    visitor.visit_expression_mut(expr)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ArrayPatternElement::SingleNameRest { ident } => visitor.visit_identifier_mut(ident),
            ArrayPatternElement::PatternRest { pattern } => visitor.visit_pattern_mut(pattern),
            ArrayPatternElement::Elision => {
                // special case to be handled by user
                ControlFlow::Continue(())
            }
        }
    }
}
