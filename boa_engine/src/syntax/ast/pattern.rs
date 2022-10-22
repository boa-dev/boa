use boa_interner::{Interner, ToInternedString};

use super::{
    expression::{
        access::{PropertyAccess, PropertyAccessField},
        Identifier,
    },
    property::PropertyName,
    ContainsSymbol, Expression,
};

/// `Pattern` represents an object or array pattern.
///
/// This enum mostly wraps the functionality of the specific binding pattern types.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `BindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-BindingPattern
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Object(PatternObject),
    Array(PatternArray),
}

impl From<PatternObject> for Pattern {
    fn from(obj: PatternObject) -> Self {
        Pattern::Object(obj)
    }
}

impl From<PatternArray> for Pattern {
    fn from(obj: PatternArray) -> Self {
        Pattern::Array(obj)
    }
}

impl From<Vec<PatternObjectElement>> for Pattern {
    fn from(elements: Vec<PatternObjectElement>) -> Self {
        PatternObject::new(elements.into()).into()
    }
}
impl From<Vec<PatternArrayElement>> for Pattern {
    fn from(elements: Vec<PatternArrayElement>) -> Self {
        PatternArray::new(elements.into()).into()
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

impl Pattern {
    /// Gets the list of identifiers in the pattern.
    ///
    /// A single pattern may have 0 to n identifiers.
    #[inline]
    pub fn idents(&self) -> Vec<Identifier> {
        match &self {
            Pattern::Object(pattern) => pattern.idents(),
            Pattern::Array(pattern) => pattern.idents(),
        }
    }

    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Pattern::Object(object) => object.contains_arguments(),
            Pattern::Array(array) => array.contains_arguments(),
        }
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Pattern::Object(object) => object.contains(symbol),
            Pattern::Array(array) => array.contains(symbol),
        }
    }
}

/// `DeclarationPatternObject` represents an object binding pattern.
///
/// This struct holds a list of bindings, and an optional initializer for the binding pattern.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `ObjectBindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-ObjectBindingPattern
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PatternObject(Box<[PatternObjectElement]>);

impl From<Vec<PatternObjectElement>> for PatternObject {
    fn from(elements: Vec<PatternObjectElement>) -> Self {
        Self(elements.into())
    }
}

impl ToInternedString for PatternObject {
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
        buf.push('}');
        buf
    }
}

impl PatternObject {
    /// Create a new object binding pattern.
    #[inline]
    pub(in crate::syntax) fn new(bindings: Box<[PatternObjectElement]>) -> Self {
        Self(bindings)
    }

    /// Gets the bindings for the object binding pattern.
    #[inline]
    pub(crate) fn bindings(&self) -> &[PatternObjectElement] {
        &self.0
    }

    // Returns if the object binding pattern has a rest element.
    #[inline]
    pub(crate) fn has_rest(&self) -> bool {
        matches!(
            self.0.last(),
            Some(PatternObjectElement::RestProperty { .. })
        )
    }

    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.0.iter().any(PatternObjectElement::contains_arguments)
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.0.iter().any(|e| e.contains(symbol))
    }

    /// Gets the list of identifiers declared by the object binding pattern.
    #[inline]
    pub(crate) fn idents(&self) -> Vec<Identifier> {
        self.0
            .iter()
            .flat_map(PatternObjectElement::idents)
            .collect()
    }
}

/// `DeclarationPatternArray` represents an array binding pattern.
///
/// This struct holds a list of bindings, and an optional initializer for the binding pattern.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `ArrayBindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-ArrayBindingPattern
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct PatternArray(Box<[PatternArrayElement]>);

impl From<Vec<PatternArrayElement>> for PatternArray {
    fn from(elements: Vec<PatternArrayElement>) -> Self {
        Self(elements.into())
    }
}

impl ToInternedString for PatternArray {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "[".to_owned();
        for (i, binding) in self.0.iter().enumerate() {
            if i == self.0.len() - 1 {
                match binding {
                    PatternArrayElement::Elision => {
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

impl PatternArray {
    /// Create a new array binding pattern.
    #[inline]
    pub(in crate::syntax) fn new(bindings: Box<[PatternArrayElement]>) -> Self {
        Self(bindings)
    }

    /// Gets the bindings for the array binding pattern.
    #[inline]
    pub(crate) fn bindings(&self) -> &[PatternArrayElement] {
        &self.0
    }

    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.0.iter().any(PatternArrayElement::contains_arguments)
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.0.iter().any(|e| e.contains(symbol))
    }

    /// Gets the list of identifiers declared by the array binding pattern.
    #[inline]
    pub(crate) fn idents(&self) -> Vec<Identifier> {
        self.0
            .iter()
            .flat_map(PatternArrayElement::idents)
            .collect()
    }
}

/// `BindingPatternTypeObject` represents the different types of bindings that an object binding pattern may contain.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `ObjectBindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-ObjectBindingPattern
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PatternObjectElement {
    /// Empty represents an empty object pattern e.g. `{ }`.
    Empty,

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
        name: PropertyName,
        ident: Identifier,
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
        ident: Identifier,
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
        name: PropertyName,
        access: PropertyAccess,
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
        access: PropertyAccess,
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
        name: PropertyName,
        pattern: Pattern,
        default_init: Option<Expression>,
    },
}

impl PatternObjectElement {
    /// Returns true if the element contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            PatternObjectElement::SingleName {
                name, default_init, ..
            } => {
                if let PropertyName::Computed(node) = name {
                    if node.contains_arguments() {
                        return true;
                    }
                }
                if let Some(init) = default_init {
                    if init.contains_arguments() {
                        return true;
                    }
                }
            }
            PatternObjectElement::AssignmentRestPropertyAccess { access, .. } => {
                if access.target().contains_arguments() {
                    return true;
                }
            }
            PatternObjectElement::Pattern {
                name,
                pattern,
                default_init,
            } => {
                if let PropertyName::Computed(node) = name {
                    if node.contains_arguments() {
                        return true;
                    }
                }
                if pattern.contains_arguments() {
                    return true;
                }
                if let Some(init) = default_init {
                    if init.contains_arguments() {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::SingleName {
                default_init: Some(node),
                ..
            } => {
                if node.contains(symbol) {
                    return true;
                }
            }
            Self::AssignmentRestPropertyAccess { access, .. } => {
                if access.target().contains(symbol) {
                    return true;
                }
            }
            Self::Pattern {
                pattern,
                default_init,
                ..
            } => {
                if let Some(node) = default_init {
                    if node.contains(symbol) {
                        return true;
                    }
                }
                if pattern.contains(symbol) {
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Gets the list of identifiers declared by the object binding pattern.
    #[inline]
    pub(crate) fn idents(&self) -> Vec<Identifier> {
        match self {
            Self::SingleName { ident, .. } | Self::RestProperty { ident, .. } => {
                vec![*ident]
            }
            Self::AssignmentPropertyAccess {
                name: PropertyName::Literal(lit),
                ..
            } => {
                vec![(*lit).into()]
            }
            Self::Pattern { pattern, .. } => pattern.idents(),
            _ => Vec::new(),
        }
    }
}

impl ToInternedString for PatternObjectElement {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Empty => String::new(),
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

/// `PatternTypeArray` represents the different types of bindings that an array binding pattern may contain.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `ArrayBindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-ArrayBindingPattern
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum PatternArrayElement {
    /// Empty represents an empty array pattern e.g. `[ ]`.
    ///
    /// This may occur because the `Elision` and `BindingRestElement` in the first type of
    /// array binding pattern are both optional.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - ArrayBindingPattern][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-ArrayBindingPattern
    Empty,

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
        ident: Identifier,
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
    PropertyAccess { access: PropertyAccess },

    /// Pattern represents a `Pattern` in an `Element` of an array pattern.
    ///
    /// The pattern and the optional default initializer are both stored in the DeclarationPattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingElement
    Pattern {
        pattern: Pattern,
        default_init: Option<Expression>,
    },

    /// SingleNameRest represents a `BindingIdentifier` in a `BindingRestElement` of an array pattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingRestElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingRestElement
    SingleNameRest { ident: Identifier },

    /// PropertyAccess represents a rest (spread operator) with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ArrayBindingPattern.
    /// This is only used when a array literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    PropertyAccessRest { access: PropertyAccess },

    /// PatternRest represents a `Pattern` in a `RestElement` of an array pattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingRestElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingRestElement
    PatternRest { pattern: Pattern },
}

impl PatternArrayElement {
    /// Returns true if the node contains a identifier reference named 'arguments'.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-containsarguments
    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            Self::SingleName {
                default_init: Some(init),
                ..
            } => {
                if init.contains_arguments() {
                    return true;
                }
            }
            Self::PropertyAccess { access } | Self::PropertyAccessRest { access } => {
                if access.target().contains_arguments() {
                    return true;
                }
                if let PropertyAccessField::Expr(expr) = access.field() {
                    if expr.contains_arguments() {
                        return true;
                    }
                }
            }
            Self::PatternRest { pattern } => {
                if pattern.contains_arguments() {
                    return true;
                }
            }
            Self::Pattern {
                pattern,
                default_init,
            } => {
                if pattern.contains_arguments() {
                    return true;
                }
                if let Some(init) = default_init {
                    if init.contains_arguments() {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::SingleName {
                default_init: Some(node),
                ..
            } => {
                if node.contains(symbol) {
                    return true;
                }
            }
            Self::PropertyAccess { access } | Self::PropertyAccessRest { access } => {
                if access.target().contains(symbol) {
                    return true;
                }

                if let PropertyAccessField::Expr(expr) = access.field() {
                    if expr.contains(symbol) {
                        return true;
                    }
                }
            }
            Self::Pattern {
                pattern,
                default_init,
            } => {
                if pattern.contains(symbol) {
                    return true;
                }

                if let Some(init) = default_init {
                    if init.contains(symbol) {
                        return true;
                    }
                }
            }
            Self::PatternRest { pattern } => {
                if pattern.contains(symbol) {
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Gets the list of identifiers in the array pattern element.
    #[inline]
    pub(crate) fn idents(&self) -> Vec<Identifier> {
        match self {
            Self::SingleName { ident, .. } => {
                vec![*ident]
            }
            Self::Pattern { pattern, .. } | Self::PatternRest { pattern } => pattern.idents(),
            Self::SingleNameRest { ident } => vec![*ident],
            _ => Vec::new(),
        }
    }
}

impl ToInternedString for PatternArrayElement {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Empty => String::new(),
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
