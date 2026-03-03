//! Variable related declarations.

use super::Declaration;
use crate::{
    Statement,
    expression::{Expression, Identifier},
    join_nodes,
    pattern::Pattern,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, ToInternedString};
use core::{convert::TryFrom, fmt::Write as _, ops::ControlFlow};

/// A [`var`][var] statement, also called [`VariableStatement`][varstmt] in the spec.
///
/// The scope of a variable declared with `var` is its current execution context, which is either
/// the enclosing function or, for variables declared outside any function, global. If you
/// re-declare a ECMAScript variable, it will not lose its value.
///
/// Although a bit confusing, `VarDeclaration`s are not considered [`Declaration`]s by the spec.
/// This is partly because it has very different semantics from `let` and `const` declarations, but
/// also because a `var` statement can be labelled just like any other [`Statement`]:
///
/// ```javascript
/// label: var a = 5;
/// a;
/// ```
///
/// returns `5` as the value of the statement list, while:
///
/// ```javascript
/// label: let a = 5;
/// a;
/// ```
/// throws a `SyntaxError`.
///
/// `var` declarations, wherever they occur, are processed before any code is executed. This is
/// called <code>[hoisting]</code>.
///
/// [var]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/var
/// [varstmt]: https://tc39.es/ecma262/#prod-VariableStatement
/// [hoisting]: https://developer.mozilla.org/en-US/docs/Glossary/Hoisting
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct VarDeclaration<'arena>(pub VariableList<'arena>);

impl<'arena> From<VarDeclaration<'arena>> for Statement<'arena> {
    fn from(var: VarDeclaration<'arena>) -> Self {
        Self::Var(var)
    }
}

impl<'arena> ToInternedString for VarDeclaration<'arena> {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!("var {}", self.0.to_interned_string(interner))
    }
}

impl<'arena> VisitWith<'arena> for VarDeclaration<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_variable_list(&self.0)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_variable_list_mut(&mut self.0)
    }
}

/// A **[lexical declaration]** defines variables that are scoped to the lexical environment of
/// the variable declaration.
///
/// [lexical declaration]: https://tc39.es/ecma262/#sec-let-and-const-declarations
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum LexicalDeclaration<'arena> {
    /// A <code>[const]</code> variable creates a constant whose scope can be either global or local
    /// to the block in which it is declared.
    ///
    /// An initializer for a constant is required. You must specify its value in the same statement
    /// in which it's declared. (This makes sense, given that it can't be changed later)
    ///
    /// [const]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const
    Const(VariableList<'arena>),

    /// A <code>[let]</code> variable is limited to a scope of a block statement, or expression on
    /// which it is used, unlike the `var` keyword, which defines a variable globally, or locally to
    /// an entire function regardless of block scope.
    ///
    /// Just like const, `let` does not create properties of the window object when declared
    /// globally (in the top-most scope).
    ///
    /// If a let declaration does not have an initializer, the variable is assigned the value `undefined`.
    ///
    /// [let]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/let
    Let(VariableList<'arena>),
}

impl<'arena> LexicalDeclaration<'arena> {
    /// Gets the inner variable list of the `LexicalDeclaration`
    #[must_use]
    pub const fn variable_list(&self) -> &VariableList<'arena> {
        match self {
            Self::Const(list) | Self::Let(list) => list,
        }
    }

    /// Returns `true` if the declaration is a `const` declaration.
    #[must_use]
    pub const fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }
}

impl<'arena> From<LexicalDeclaration<'arena>> for Declaration<'arena> {
    fn from(lex: LexicalDeclaration<'arena>) -> Self {
        Self::Lexical(lex)
    }
}

impl<'arena> ToInternedString for LexicalDeclaration<'arena> {
    fn to_interned_string(&self, interner: &Interner) -> String {
        format!(
            "{} {}",
            match &self {
                Self::Let(_) => "let",
                Self::Const(_) => "const",
            },
            self.variable_list().to_interned_string(interner)
        )
    }
}

impl<'arena> VisitWith<'arena> for LexicalDeclaration<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        match self {
            Self::Const(vars) | Self::Let(vars) => visitor.visit_variable_list(vars),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        match self {
            Self::Const(vars) | Self::Let(vars) => visitor.visit_variable_list_mut(vars),
        }
    }
}

/// List of variables in a variable declaration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct VariableList<'arena> {
    list: Box<[Variable<'arena>]>,
}

impl<'arena> VariableList<'arena> {
    /// Creates a variable list if the provided list of [`Variable`] is not empty.
    #[must_use]
    pub fn new(list: Box<[Variable<'arena>]>) -> Option<Self> {
        if list.is_empty() {
            return None;
        }

        Some(Self { list })
    }
}

impl<'arena> AsRef<[Variable<'arena>]> for VariableList<'arena> {
    fn as_ref(&self) -> &[Variable<'arena>] {
        &self.list
    }
}

impl<'arena> ToInternedString for VariableList<'arena> {
    fn to_interned_string(&self, interner: &Interner) -> String {
        join_nodes(interner, self.list.as_ref())
    }
}

impl<'arena> VisitWith<'arena> for VariableList<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        for variable in &*self.list {
            visitor.visit_variable(variable)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        for variable in &mut *self.list {
            visitor.visit_variable_mut(variable)?;
        }
        ControlFlow::Continue(())
    }
}

/// The error returned by the [`VariableList<'arena>::try_from`] function.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromVariableListError(());

impl std::fmt::Display for TryFromVariableListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "provided list of variables cannot be empty".fmt(f)
    }
}

impl<'arena> TryFrom<Box<[Variable<'arena>]>> for VariableList<'arena> {
    type Error = TryFromVariableListError;

    fn try_from(value: Box<[Variable<'arena>]>) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(TryFromVariableListError(()))
    }
}

impl<'arena> TryFrom<Vec<Variable<'arena>>> for VariableList<'arena> {
    type Error = TryFromVariableListError;

    fn try_from(value: Vec<Variable<'arena>>) -> Result<Self, Self::Error> {
        Self::try_from(value.into_boxed_slice())
    }
}

/// Variable represents a variable declaration of some kind.
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct Variable<'arena> {
    binding: Binding<'arena>,
    init: Option<Expression<'arena>>,
}

impl<'arena> ToInternedString for Variable<'arena> {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = self.binding.to_interned_string(interner);

        if let Some(ref init) = self.init {
            let _ = write!(buf, " = {}", init.to_interned_string(interner));
        }
        buf
    }
}

impl<'arena> Variable<'arena> {
    /// Creates a new variable declaration from a `BindingIdentifier`.
    #[inline]
    #[must_use]
    pub const fn from_identifier(ident: Identifier, init: Option<Expression<'arena>>) -> Self {
        Self {
            binding: Binding::Identifier(ident),
            init,
        }
    }

    /// Creates a new variable declaration from a `Pattern`.
    #[inline]
    #[must_use]
    pub const fn from_pattern(pattern: Pattern<'arena>, init: Option<Expression<'arena>>) -> Self {
        Self {
            binding: Binding::Pattern(pattern),
            init,
        }
    }
    /// Gets the variable declaration binding.
    #[must_use]
    pub const fn binding(&self) -> &Binding<'arena> {
        &self.binding
    }

    /// Gets the initialization expression for the variable declaration, if any.
    #[inline]
    #[must_use]
    pub const fn init(&self) -> Option<&Expression<'arena>> {
        self.init.as_ref()
    }
}

impl<'arena> VisitWith<'arena> for Variable<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_binding(&self.binding)?;
        if let Some(init) = &self.init {
            visitor.visit_expression(init)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_binding_mut(&mut self.binding)?;
        if let Some(init) = &mut self.init {
            visitor.visit_expression_mut(init)?;
        }
        ControlFlow::Continue(())
    }
}

/// Binding represents either an individual binding or a binding pattern.
///
/// More information:
///  - [ECMAScript reference: 14.3 Declarations and the Variable Statement][spec]
///
/// [spec]:  https://tc39.es/ecma262/#sec-declarations-and-the-variable-statement
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum Binding<'arena> {
    /// A single identifier binding.
    Identifier(Identifier),
    /// A pattern binding.
    Pattern(Pattern<'arena>),
}

impl<'arena> From<Identifier> for Binding<'arena> {
    fn from(id: Identifier) -> Self {
        Self::Identifier(id)
    }
}

impl<'arena> From<Pattern<'arena>> for Binding<'arena> {
    fn from(pat: Pattern<'arena>) -> Self {
        Self::Pattern(pat)
    }
}

impl<'arena> ToInternedString for Binding<'arena> {
    fn to_interned_string(&self, interner: &Interner) -> String {
        match self {
            Self::Identifier(id) => id.to_interned_string(interner),
            Self::Pattern(pattern) => pattern.to_interned_string(interner),
        }
    }
}

impl<'arena> VisitWith<'arena> for Binding<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        match self {
            Self::Identifier(id) => visitor.visit_identifier(id),
            Self::Pattern(pattern) => visitor.visit_pattern(pattern),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        match self {
            Self::Identifier(id) => visitor.visit_identifier_mut(id),
            Self::Pattern(pattern) => visitor.visit_pattern_mut(pattern),
        }
    }
}
