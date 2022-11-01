use crate::{
    declaration::{Binding, Variable},
    expression::{Expression, Identifier},
    pattern::Pattern,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    ContainsSymbol,
};
use bitflags::bitflags;
use boa_interner::{Interner, Sym, ToInternedString};
use core::ops::ControlFlow;
use rustc_hash::FxHashSet;

/// A list of `FormalParameter`s that describes the parameters of a function, as defined by the [spec].
///
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameterList
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FormalParameterList {
    parameters: Box<[FormalParameter]>,
    flags: FormalParameterListFlags,
    length: u32,
}

impl FormalParameterList {
    /// Creates a new empty formal parameter list.
    pub fn new() -> Self {
        Self {
            parameters: Box::new([]),
            flags: FormalParameterListFlags::default(),
            length: 0,
        }
    }

    /// Creates a `FormalParameterList` from a list of [`FormalParameter`]s.
    pub fn from_parameters(parameters: Vec<FormalParameter>) -> Self {
        let mut flags = FormalParameterListFlags::default();
        let mut length = 0;
        let mut names = FxHashSet::default();

        for parameter in &parameters {
            let parameter_names = parameter.names();

            for name in parameter_names {
                if name == Sym::ARGUMENTS {
                    flags |= FormalParameterListFlags::HAS_ARGUMENTS;
                }
                if names.contains(&name) {
                    flags |= FormalParameterListFlags::HAS_DUPLICATES;
                } else {
                    names.insert(name);
                }
            }

            if parameter.is_rest_param() {
                flags |= FormalParameterListFlags::HAS_REST_PARAMETER;
            }
            if parameter.init().is_some() {
                flags |= FormalParameterListFlags::HAS_EXPRESSIONS;
            }
            if parameter.is_rest_param() || parameter.init().is_some() || !parameter.is_identifier()
            {
                flags.remove(FormalParameterListFlags::IS_SIMPLE);
            }
            if !(flags.contains(FormalParameterListFlags::HAS_EXPRESSIONS)
                || parameter.is_rest_param()
                || parameter.init().is_some())
            {
                length += 1;
            }
        }

        Self {
            parameters: parameters.into(),
            flags,
            length,
        }
    }

    /// Returns the length of the parameter list.
    /// Note that this is not equal to the length of the parameters slice.
    pub fn length(&self) -> u32 {
        self.length
    }

    /// Returns the parameter list flags.
    pub fn flags(&self) -> FormalParameterListFlags {
        self.flags
    }

    /// Indicates if the parameter list is simple.
    pub fn is_simple(&self) -> bool {
        self.flags.contains(FormalParameterListFlags::IS_SIMPLE)
    }

    /// Indicates if the parameter list has duplicate parameters.
    pub fn has_duplicates(&self) -> bool {
        self.flags
            .contains(FormalParameterListFlags::HAS_DUPLICATES)
    }

    /// Indicates if the parameter list has a rest parameter.
    pub fn has_rest_parameter(&self) -> bool {
        self.flags
            .contains(FormalParameterListFlags::HAS_REST_PARAMETER)
    }

    /// Indicates if the parameter list has expressions in it's parameters.
    pub fn has_expressions(&self) -> bool {
        self.flags
            .contains(FormalParameterListFlags::HAS_EXPRESSIONS)
    }

    /// Indicates if the parameter list has parameters named 'arguments'.
    pub fn has_arguments(&self) -> bool {
        self.flags.contains(FormalParameterListFlags::HAS_ARGUMENTS)
    }

    /// Check if the any of the parameters contains a yield expression.
    pub fn contains_yield_expression(&self) -> bool {
        for parameter in self.parameters.iter() {
            if parameter
                .variable()
                .contains(ContainsSymbol::YieldExpression)
            {
                return true;
            }
        }
        false
    }

    /// Check if the any of the parameters contains a await expression.
    pub fn contains_await_expression(&self) -> bool {
        for parameter in self.parameters.iter() {
            if parameter
                .variable()
                .contains(ContainsSymbol::AwaitExpression)
            {
                return true;
            }
        }
        false
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.parameters
            .iter()
            .any(FormalParameter::contains_arguments)
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.parameters.iter().any(|param| param.contains(symbol))
    }
}

impl From<Vec<FormalParameter>> for FormalParameterList {
    fn from(parameters: Vec<FormalParameter>) -> Self {
        Self::from_parameters(parameters)
    }
}

impl From<FormalParameter> for FormalParameterList {
    fn from(parameter: FormalParameter) -> Self {
        Self::from_parameters(vec![parameter])
    }
}

impl AsRef<[FormalParameter]> for FormalParameterList {
    fn as_ref(&self) -> &[FormalParameter] {
        &self.parameters
    }
}

impl VisitWith for FormalParameterList {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for parameter in self.parameters.iter() {
            try_break!(visitor.visit_formal_parameter(parameter));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for parameter in self.parameters.iter_mut() {
            try_break!(visitor.visit_formal_parameter_mut(parameter));
        }
        // TODO recompute flags
        ControlFlow::Continue(())
    }
}

bitflags! {
    /// Flags for a [`FormalParameterList`].
    #[allow(clippy::unsafe_derive_deserialize)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct FormalParameterListFlags: u8 {
        /// Has only identifier parameters with no initialization expressions.
        const IS_SIMPLE = 0b0000_0001;
        /// Has any duplicate parameters.
        const HAS_DUPLICATES = 0b0000_0010;
        /// Has a rest parameter.
        const HAS_REST_PARAMETER = 0b0000_0100;
        /// Has any initialization expression.
        const HAS_EXPRESSIONS = 0b0000_1000;
        /// Has an argument with the name `arguments`.
        const HAS_ARGUMENTS = 0b0001_0000;
    }
}

impl Default for FormalParameterListFlags {
    fn default() -> Self {
        Self::empty().union(Self::IS_SIMPLE)
    }
}

/// "Formal parameter" is a fancy way of saying "function parameter".
///
/// In the declaration of a function, the parameters must be identifiers,
/// not any value like numbers, strings, or objects.
/// ```text
/// function foo(formalParameter1, formalParameter2) {
/// }
/// ```
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameter
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors/Missing_formal_parameter
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct FormalParameter {
    variable: Variable,
    is_rest_param: bool,
}

impl FormalParameter {
    /// Creates a new formal parameter.
    pub fn new<D>(variable: D, is_rest_param: bool) -> Self
    where
        D: Into<Variable>,
    {
        Self {
            variable: variable.into(),
            is_rest_param,
        }
    }

    /// Gets the name of the formal parameter.
    pub fn names(&self) -> Vec<Identifier> {
        match self.variable.binding() {
            Binding::Identifier(ident) => vec![*ident],
            Binding::Pattern(pattern) => match pattern {
                Pattern::Object(object_pattern) => object_pattern.idents(),

                Pattern::Array(array_pattern) => array_pattern.idents(),
            },
        }
    }

    /// Gets the variable of the formal parameter
    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    /// Gets the initialization node of the formal parameter, if any.
    pub fn init(&self) -> Option<&Expression> {
        self.variable.init()
    }

    /// Returns `true` if the parameter is a rest parameter.
    pub fn is_rest_param(&self) -> bool {
        self.is_rest_param
    }

    /// Returns `true` if the parameter is a simple [`Identifier`].
    pub fn is_identifier(&self) -> bool {
        matches!(&self.variable.binding(), Binding::Identifier(_))
    }

    pub(crate) fn contains_arguments(&self) -> bool {
        self.variable.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.variable.contains(symbol)
    }
}

impl ToInternedString for FormalParameter {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = if self.is_rest_param {
            "...".to_owned()
        } else {
            String::new()
        };
        buf.push_str(&self.variable.to_interned_string(interner));
        buf
    }
}

impl VisitWith for FormalParameter {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        visitor.visit_variable(&self.variable)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        visitor.visit_variable_mut(&mut self.variable)
    }
}
