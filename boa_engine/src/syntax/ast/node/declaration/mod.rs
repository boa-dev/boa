//! Declaration nodes
use crate::syntax::ast::node::{
    field::{GetConstField, GetField},
    join_nodes,
    object::PropertyName,
    statement_list::StatementList,
    ContainsSymbol, Identifier, Node,
};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

pub mod arrow_function_decl;
pub mod async_function_decl;
pub mod async_function_expr;
pub mod async_generator_decl;
pub mod async_generator_expr;
pub mod class_decl;
pub mod function_decl;
pub mod function_expr;
pub mod generator_decl;
pub mod generator_expr;

pub use self::{
    arrow_function_decl::ArrowFunctionDecl, async_function_decl::AsyncFunctionDecl,
    async_function_expr::AsyncFunctionExpr, async_generator_decl::AsyncGeneratorDecl,
    async_generator_expr::AsyncGeneratorExpr, function_decl::FunctionDecl,
    function_expr::FunctionExpr,
};

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationList {
    /// The `const` statements are block-scoped, much like variables defined using the `let`
    /// keyword.
    ///
    /// This declaration creates a constant whose scope can be either global or local to the block
    /// in which it is declared. Global constants do not become properties of the window object,
    /// unlike var variables.
    ///
    /// An initializer for a constant is required. You must specify its value in the same statement
    /// in which it's declared. (This makes sense, given that it can't be changed later.)
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
    /// [identifier]: https://developer.mozilla.org/en-US/docs/Glossary/identifier
    /// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
    Const(Box<[Declaration]>),

    /// The `let` statement declares a block scope local variable, optionally initializing it to a
    /// value.
    ///
    ///
    /// `let` allows you to declare variables that are limited to a scope of a block statement, or
    /// expression on which it is used, unlike the `var` keyword, which defines a variable
    /// globally, or locally to an entire function regardless of block scope.
    ///
    /// Just like const the `let` does not create properties of the window object when declared
    /// globally (in the top-most scope).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-let-and-const-declarations
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let
    Let(Box<[Declaration]>),

    /// The `var` statement declares a variable, optionally initializing it to a value.
    ///
    /// var declarations, wherever they occur, are processed before any code is executed. This is
    /// called hoisting, and is discussed further below.
    ///
    /// The scope of a variable declared with var is its current execution context, which is either
    /// the enclosing function or, for variables declared outside any function, global. If you
    /// re-declare a JavaScript variable, it will not lose its value.
    ///
    /// Assigning a value to an undeclared variable implicitly creates it as a global variable (it
    /// becomes a property of the global object) when the assignment is executed.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-VariableStatement
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
    Var(Box<[Declaration]>),
}

impl AsRef<[Declaration]> for DeclarationList {
    fn as_ref(&self) -> &[Declaration] {
        use DeclarationList::{Const, Let, Var};
        match self {
            Var(list) | Const(list) | Let(list) => list,
        }
    }
}

impl ToInternedString for DeclarationList {
    fn to_interned_string(&self, interner: &Interner) -> String {
        if self.as_ref().is_empty() {
            String::new()
        } else {
            use DeclarationList::{Const, Let, Var};
            format!(
                "{} {}",
                match &self {
                    Let(_) => "let",
                    Const(_) => "const",
                    Var(_) => "var",
                },
                join_nodes(interner, self.as_ref())
            )
        }
    }
}

impl From<DeclarationList> for Node {
    fn from(list: DeclarationList) -> Self {
        use DeclarationList::{Const, Let, Var};
        match &list {
            Let(_) => Self::LetDeclList(list),
            Const(_) => Self::ConstDeclList(list),
            Var(_) => Self::VarDeclList(list),
        }
    }
}

impl From<Declaration> for Box<[Declaration]> {
    fn from(d: Declaration) -> Self {
        Box::new([d])
    }
}

/// Declaration represents either an individual binding or a binding pattern.
///
/// For `let` and `const` declarations this type represents a [`LexicalBinding`][spec1]
///
/// For `var` declarations this type represents a [`VariableDeclaration`][spec2]
///
/// More information:
///  - [ECMAScript reference: 14.3 Declarations and the Variable Statement][spec3]
///
/// [spec1]: https://tc39.es/ecma262/#prod-LexicalBinding
/// [spec2]: https://tc39.es/ecma262/#prod-VariableDeclaration
/// [spec3]:  https://tc39.es/ecma262/#sec-declarations-and-the-variable-statement
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    Identifier {
        ident: Identifier,
        init: Option<Node>,
    },
    Pattern(DeclarationPattern),
}

impl ToInternedString for Declaration {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match &self {
            Self::Identifier { ident, init } => {
                let mut buf = ident.to_interned_string(interner);
                if let Some(ref init) = &init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
            Self::Pattern(pattern) => pattern.to_interned_string(interner),
        }
    }
}

impl Declaration {
    /// Creates a new variable declaration with a `BindingIdentifier`.
    #[inline]
    pub(in crate::syntax) fn new_with_identifier<N, I>(ident: N, init: I) -> Self
    where
        N: Into<Identifier>,
        I: Into<Option<Node>>,
    {
        Self::Identifier {
            ident: ident.into(),
            init: init.into(),
        }
    }

    /// Creates a new variable declaration with an `ObjectBindingPattern`.
    #[inline]
    pub(in crate::syntax) fn new_with_object_pattern<I>(
        bindings: Vec<BindingPatternTypeObject>,
        init: I,
    ) -> Self
    where
        I: Into<Option<Node>>,
    {
        Self::Pattern(DeclarationPattern::Object(DeclarationPatternObject::new(
            bindings,
            init.into(),
        )))
    }

    /// Creates a new variable declaration with an `ArrayBindingPattern`.
    #[inline]
    pub(in crate::syntax) fn new_with_array_pattern<I>(
        bindings: Vec<BindingPatternTypeArray>,
        init: I,
    ) -> Self
    where
        I: Into<Option<Node>>,
    {
        Self::Pattern(DeclarationPattern::Array(DeclarationPatternArray::new(
            bindings,
            init.into(),
        )))
    }

    /// Gets the initialization node for the declaration, if any.
    #[inline]
    pub(crate) fn init(&self) -> Option<&Node> {
        match &self {
            Self::Identifier { init, .. } => init.as_ref(),
            Self::Pattern(pattern) => pattern.init(),
        }
    }

    /// Returns `true` if the node contains the given token.
    ///
    /// More information:
    ///  - [ECMAScript specification][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-contains
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            Self::Identifier { init, .. } => {
                if let Some(node) = init {
                    if node.contains(symbol) {
                        return true;
                    }
                }
            }
            Self::Pattern(pattern) => {
                if pattern.contains(symbol) {
                    return true;
                }
            }
        }
        false
    }
}

/// `DeclarationPattern` represents an object or array binding pattern.
///
/// This enum mostly wraps the functionality of the specific binding pattern types.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `BindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-BindingPattern
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationPattern {
    Object(DeclarationPatternObject),
    Array(DeclarationPatternArray),
}

impl ToInternedString for DeclarationPattern {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match &self {
            DeclarationPattern::Object(o) => o.to_interned_string(interner),
            DeclarationPattern::Array(a) => a.to_interned_string(interner),
        }
    }
}

impl DeclarationPattern {
    /// Gets the list of identifiers declared by the binding pattern.
    ///
    /// A single binding pattern may declare 0 to n identifiers.
    #[inline]
    pub fn idents(&self) -> Vec<Sym> {
        match &self {
            DeclarationPattern::Object(pattern) => pattern.idents(),
            DeclarationPattern::Array(pattern) => pattern.idents(),
        }
    }

    /// Gets the initialization node for the binding pattern, if any.
    #[inline]
    pub fn init(&self) -> Option<&Node> {
        match &self {
            DeclarationPattern::Object(pattern) => pattern.init(),
            DeclarationPattern::Array(pattern) => pattern.init(),
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
            DeclarationPattern::Object(pattern) => {
                if let Some(init) = pattern.init() {
                    if init.contains_arguments() {
                        return true;
                    }
                }
                for binding in pattern.bindings() {
                    match binding {
                        BindingPatternTypeObject::SingleName {
                            property_name,
                            default_init,
                            ..
                        } => {
                            if let PropertyName::Computed(node) = property_name {
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
                        BindingPatternTypeObject::RestGetConstField {
                            get_const_field, ..
                        } => {
                            if get_const_field.obj().contains_arguments() {
                                return true;
                            }
                        }
                        BindingPatternTypeObject::BindingPattern {
                            ident,
                            pattern,
                            default_init,
                        } => {
                            if let PropertyName::Computed(node) = ident {
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
                }
            }
            DeclarationPattern::Array(pattern) => {
                if let Some(init) = pattern.init() {
                    if init.contains_arguments() {
                        return true;
                    }
                }
                for binding in pattern.bindings() {
                    match binding {
                        BindingPatternTypeArray::SingleName {
                            default_init: Some(init),
                            ..
                        } => {
                            if init.contains_arguments() {
                                return true;
                            }
                        }
                        BindingPatternTypeArray::GetField { get_field }
                        | BindingPatternTypeArray::GetFieldRest { get_field } => {
                            if get_field.obj().contains_arguments() {
                                return true;
                            }
                            if get_field.field().contains_arguments() {
                                return true;
                            }
                        }
                        BindingPatternTypeArray::GetConstField { get_const_field }
                        | BindingPatternTypeArray::GetConstFieldRest { get_const_field } => {
                            if get_const_field.obj().contains_arguments() {
                                return true;
                            }
                        }
                        BindingPatternTypeArray::BindingPattern { pattern }
                        | BindingPatternTypeArray::BindingPatternRest { pattern } => {
                            if pattern.contains_arguments() {
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
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
            DeclarationPattern::Object(object) => {
                if let Some(node) = object.init() {
                    if node.contains(symbol) {
                        return true;
                    }
                }
                for binding in &object.bindings {
                    match binding {
                        BindingPatternTypeObject::SingleName {
                            default_init: Some(node),
                            ..
                        } => {
                            if node.contains(symbol) {
                                return true;
                            }
                        }
                        BindingPatternTypeObject::RestGetConstField {
                            get_const_field, ..
                        } => {
                            if get_const_field.obj().contains(symbol) {
                                return true;
                            }
                        }
                        BindingPatternTypeObject::BindingPattern {
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
                }
            }
            DeclarationPattern::Array(array) => {
                if let Some(node) = array.init() {
                    if node.contains(symbol) {
                        return true;
                    }
                }
                for binding in array.bindings() {
                    match binding {
                        BindingPatternTypeArray::SingleName {
                            default_init: Some(node),
                            ..
                        } => {
                            if node.contains(symbol) {
                                return true;
                            }
                        }
                        BindingPatternTypeArray::GetField { get_field }
                        | BindingPatternTypeArray::GetFieldRest { get_field } => {
                            if get_field.obj().contains(symbol)
                                || get_field.field().contains(symbol)
                            {
                                return true;
                            }
                        }
                        BindingPatternTypeArray::GetConstField { get_const_field }
                        | BindingPatternTypeArray::GetConstFieldRest { get_const_field } => {
                            if get_const_field.obj().contains(symbol) {
                                return true;
                            }
                        }
                        BindingPatternTypeArray::BindingPattern { pattern }
                        | BindingPatternTypeArray::BindingPatternRest { pattern } => {
                            if pattern.contains(symbol) {
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        false
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DeclarationPatternObject {
    bindings: Vec<BindingPatternTypeObject>,
    init: Option<Node>,
}

impl ToInternedString for DeclarationPatternObject {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "{".to_owned();
        for (i, binding) in self.bindings.iter().enumerate() {
            let binding = binding.to_interned_string(interner);
            let str = if i == self.bindings.len() - 1 {
                format!("{binding} ")
            } else {
                format!("{binding},")
            };

            buf.push_str(&str);
        }
        buf.push('}');
        if let Some(ref init) = self.init {
            buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
        }
        buf
    }
}

impl DeclarationPatternObject {
    /// Create a new object binding pattern.
    #[inline]
    pub(in crate::syntax) fn new(
        bindings: Vec<BindingPatternTypeObject>,
        init: Option<Node>,
    ) -> Self {
        Self { bindings, init }
    }

    /// Gets the initialization node for the object binding pattern, if any.
    #[inline]
    pub(crate) fn init(&self) -> Option<&Node> {
        self.init.as_ref()
    }

    /// Gets the bindings for the object binding pattern.
    #[inline]
    pub(crate) fn bindings(&self) -> &Vec<BindingPatternTypeObject> {
        &self.bindings
    }

    /// Gets the list of identifiers declared by the object binding pattern.
    #[inline]
    pub(crate) fn idents(&self) -> Vec<Sym> {
        let mut idents = Vec::new();

        for binding in &self.bindings {
            use BindingPatternTypeObject::{
                BindingPattern, Empty, RestGetConstField, RestProperty, SingleName,
            };

            match binding {
                Empty | RestGetConstField { .. } => {}
                SingleName {
                    ident,
                    property_name: _,
                    default_init: _,
                } => {
                    idents.push(*ident);
                }
                RestProperty {
                    ident: property_name,
                    excluded_keys: _,
                } => {
                    idents.push(*property_name);
                }
                BindingPattern {
                    ident: _,
                    pattern,
                    default_init: _,
                } => {
                    for ident in pattern.idents() {
                        idents.push(ident);
                    }
                }
            }
        }

        idents
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct DeclarationPatternArray {
    bindings: Vec<BindingPatternTypeArray>,
    init: Option<Node>,
}

impl ToInternedString for DeclarationPatternArray {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "[".to_owned();
        for (i, binding) in self.bindings.iter().enumerate() {
            if i == self.bindings.len() - 1 {
                match binding {
                    BindingPatternTypeArray::Elision => {
                        buf.push_str(&format!("{}, ", binding.to_interned_string(interner)));
                    }
                    _ => buf.push_str(&format!("{} ", binding.to_interned_string(interner))),
                }
            } else {
                buf.push_str(&format!("{},", binding.to_interned_string(interner)));
            }
        }
        buf.push(']');
        if let Some(ref init) = self.init {
            buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
        }
        buf
    }
}

impl DeclarationPatternArray {
    /// Create a new array binding pattern.
    #[inline]
    pub(in crate::syntax) fn new(
        bindings: Vec<BindingPatternTypeArray>,
        init: Option<Node>,
    ) -> Self {
        Self { bindings, init }
    }

    /// Gets the initialization node for the array binding pattern, if any.
    #[inline]
    pub(crate) fn init(&self) -> Option<&Node> {
        self.init.as_ref()
    }

    /// Gets the bindings for the array binding pattern.
    #[inline]
    pub(crate) fn bindings(&self) -> &Vec<BindingPatternTypeArray> {
        &self.bindings
    }

    /// Gets the list of identifiers declared by the array binding pattern.
    #[inline]
    pub(crate) fn idents(&self) -> Vec<Sym> {
        let mut idents = Vec::new();

        for binding in &self.bindings {
            use BindingPatternTypeArray::{
                BindingPattern, BindingPatternRest, Elision, Empty, GetConstField,
                GetConstFieldRest, GetField, GetFieldRest, SingleName, SingleNameRest,
            };

            match binding {
                Empty
                | Elision
                | GetField { .. }
                | GetConstField { .. }
                | GetFieldRest { .. }
                | GetConstFieldRest { .. } => {}
                SingleName {
                    ident,
                    default_init: _,
                } => {
                    idents.push(*ident);
                }
                BindingPattern { pattern } | BindingPatternRest { pattern } => {
                    let mut i = pattern.idents();
                    idents.append(&mut i);
                }
                SingleNameRest { ident } => idents.push(*ident),
            }
        }

        idents
    }
}

/// `BindingPatternTypeObject` represents the different types of bindings that an object binding pattern may contain.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `ObjectBindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-ObjectBindingPattern
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum BindingPatternTypeObject {
    /// Empty represents an empty object binding pattern e.g. `{ }`.
    Empty,

    /// SingleName represents one of the following properties:
    ///
    /// - `SingleNameBinding` with an identifier and an optional default initializer.
    /// - `BindingProperty` with an property name and a `SingleNameBinding` as  the `BindingElement`.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - SingleNameBinding][spec1]
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingProperty][spec2]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-SingleNameBinding
    /// [spec2]: https://tc39.es/ecma262/#prod-BindingProperty
    SingleName {
        ident: Sym,
        property_name: PropertyName,
        default_init: Option<Node>,
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
    RestProperty { ident: Sym, excluded_keys: Vec<Sym> },

    /// RestGetConstField represents a rest property (spread operator) with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ObjectBindingPattern.
    /// This is only used when a object literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    RestGetConstField {
        get_const_field: GetConstField,
        excluded_keys: Vec<Sym>,
    },

    /// BindingPattern represents a `BindingProperty` with a `BindingPattern` as the `BindingElement`.
    ///
    /// Additionally to the identifier of the new property and the nested binding pattern,
    /// this may also include an optional default initializer.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingProperty][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingProperty
    BindingPattern {
        ident: PropertyName,
        pattern: DeclarationPattern,
        default_init: Option<Node>,
    },
}

impl ToInternedString for BindingPatternTypeObject {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Empty => String::new(),
            Self::SingleName {
                ident,
                property_name,
                default_init,
            } => {
                let mut buf = match property_name {
                    PropertyName::Literal(name) if *name == *ident => {
                        format!(" {}", interner.resolve_expect(*ident))
                    }
                    PropertyName::Literal(name) => {
                        format!(
                            " {} : {}",
                            interner.resolve_expect(*name),
                            interner.resolve_expect(*ident)
                        )
                    }
                    PropertyName::Computed(node) => {
                        format!(
                            " [{}] : {}",
                            node.to_interned_string(interner),
                            interner.resolve_expect(*ident)
                        )
                    }
                };
                if let Some(ref init) = default_init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
            Self::RestProperty {
                ident: property_name,
                excluded_keys: _,
            } => {
                format!(" ... {}", interner.resolve_expect(*property_name))
            }
            Self::RestGetConstField {
                get_const_field, ..
            } => {
                format!(" ... {}", get_const_field.to_interned_string(interner))
            }
            Self::BindingPattern {
                ident: property_name,
                pattern,
                default_init,
            } => {
                let mut buf = match property_name {
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

/// `BindingPatternTypeArray` represents the different types of bindings that an array binding pattern may contain.
///
/// More information:
///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - `ArrayBindingPattern`][spec1]
///
/// [spec1]: https://tc39.es/ecma262/#prod-ArrayBindingPattern
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum BindingPatternTypeArray {
    /// Empty represents an empty array binding pattern e.g. `[ ]`.
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

    /// SingleName represents a `SingleNameBinding` with an identifier and an optional default initializer.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - SingleNameBinding][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-SingleNameBinding
    SingleName {
        ident: Sym,
        default_init: Option<Node>,
    },

    /// GetField represents a binding with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ArrayBindingPattern.
    /// This is only used when a array literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    GetField { get_field: GetField },

    /// GetConstField represents a binding with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ArrayBindingPattern.
    /// This is only used when a array literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    GetConstField { get_const_field: GetConstField },

    /// BindingPattern represents a `BindingPattern` in a `BindingElement` of an array binding pattern.
    ///
    /// The pattern and the optional default initializer are both stored in the DeclarationPattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingElement
    BindingPattern { pattern: DeclarationPattern },

    /// SingleNameRest represents a `BindingIdentifier` in a `BindingRestElement` of an array binding pattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingRestElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingRestElement
    SingleNameRest { ident: Sym },

    /// GetFieldRest represents a rest binding (spread operator) with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ArrayBindingPattern.
    /// This is only used when a array literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    GetFieldRest { get_field: GetField },

    /// GetConstFieldRest represents a rest binding (spread operator) with a property accessor.
    ///
    /// Note: According to the spec this is not part of an ArrayBindingPattern.
    /// This is only used when a array literal is used as the left-hand-side of an assignment expression.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#prod-AssignmentExpression
    GetConstFieldRest { get_const_field: GetConstField },

    /// SingleNameRest represents a `BindingPattern` in a `BindingRestElement` of an array binding pattern.
    ///
    /// More information:
    ///  - [ECMAScript reference: 14.3.3 Destructuring Binding Patterns - BindingRestElement][spec1]
    ///
    /// [spec1]: https://tc39.es/ecma262/#prod-BindingRestElement
    BindingPatternRest { pattern: DeclarationPattern },
}

impl ToInternedString for BindingPatternTypeArray {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Elision => " ".to_owned(),
            Self::SingleName {
                ident,
                default_init,
            } => {
                let mut buf = format!(" {}", interner.resolve_expect(*ident));
                if let Some(ref init) = default_init {
                    buf.push_str(&format!(" = {}", init.to_interned_string(interner)));
                }
                buf
            }
            Self::GetField { get_field } => {
                format!(" {}", get_field.to_interned_string(interner))
            }
            Self::GetConstField { get_const_field } => {
                format!(" {}", get_const_field.to_interned_string(interner))
            }
            Self::BindingPattern { pattern } => {
                format!(" {}", pattern.to_interned_string(interner))
            }
            Self::SingleNameRest { ident } => {
                format!(" ... {}", interner.resolve_expect(*ident))
            }
            Self::GetFieldRest { get_field } => {
                format!(" ... {}", get_field.to_interned_string(interner))
            }
            Self::GetConstFieldRest { get_const_field } => {
                format!(" ... {}", get_const_field.to_interned_string(interner))
            }
            Self::BindingPatternRest { pattern } => {
                format!(" ... {}", pattern.to_interned_string(interner))
            }
        }
    }
}

/// Displays the body of a block or statement list.
///
/// This includes the curly braces at the start and end. This will not indent the first brace,
/// but will indent the last brace.
pub(in crate::syntax::ast::node) fn block_to_string(
    body: &StatementList,
    interner: &Interner,
    indentation: usize,
) -> String {
    if body.items().is_empty() {
        "{}".to_owned()
    } else {
        format!(
            "{{\n{}{}}}",
            body.to_indented_string(interner, indentation + 1),
            "    ".repeat(indentation)
        )
    }
}
