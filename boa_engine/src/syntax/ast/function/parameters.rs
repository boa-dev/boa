use crate::syntax::{
    ast::{
        declaration::{Binding, Variable},
        expression::{Expression, Identifier},
        pattern::Pattern,
        ContainsSymbol, Position,
    },
    parser::ParseError,
};
use bitflags::bitflags;
use boa_interner::{Interner, Sym, ToInternedString};
use rustc_hash::FxHashSet;

/// A list of `FormalParameter`s that describes the parameters of a function, as defined by the [spec].
///
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameterList
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FormalParameterList {
    pub(crate) parameters: Box<[FormalParameter]>,
    pub(crate) flags: FormalParameterListFlags,
    pub(crate) length: u32,
}

impl FormalParameterList {
    /// Creates a new formal parameter list.
    pub(crate) fn new(
        parameters: Box<[FormalParameter]>,
        flags: FormalParameterListFlags,
        length: u32,
    ) -> Self {
        Self {
            parameters,
            flags,
            length,
        }
    }

    /// Creates a new empty formal parameter list.
    pub(crate) fn empty() -> Self {
        Self {
            parameters: Box::new([]),
            flags: FormalParameterListFlags::default(),
            length: 0,
        }
    }

    /// Returns the length of the parameter list.
    /// Note that this is not equal to the length of the parameters slice.
    pub(crate) fn length(&self) -> u32 {
        self.length
    }

    /// Indicates if the parameter list is simple.
    pub(crate) fn is_simple(&self) -> bool {
        self.flags.contains(FormalParameterListFlags::IS_SIMPLE)
    }

    /// Indicates if the parameter list has duplicate parameters.
    pub(crate) fn has_duplicates(&self) -> bool {
        self.flags
            .contains(FormalParameterListFlags::HAS_DUPLICATES)
    }

    /// Indicates if the parameter list has a rest parameter.
    pub(crate) fn has_rest_parameter(&self) -> bool {
        self.flags
            .contains(FormalParameterListFlags::HAS_REST_PARAMETER)
    }

    /// Indicates if the parameter list has expressions in it's parameters.
    pub(crate) fn has_expressions(&self) -> bool {
        self.flags
            .contains(FormalParameterListFlags::HAS_EXPRESSIONS)
    }

    /// Indicates if the parameter list has parameters named 'arguments'.
    pub(crate) fn has_arguments(&self) -> bool {
        self.flags.contains(FormalParameterListFlags::HAS_ARGUMENTS)
    }

    /// Helper to check if any parameter names are declared in the given list.
    pub(crate) fn name_in_lexically_declared_names(
        &self,
        names: &[Identifier],
        position: Position,
    ) -> Result<(), ParseError> {
        for parameter in self.parameters.iter() {
            for name in &parameter.names() {
                if names.contains(name) {
                    return Err(ParseError::General {
                        message: "formal parameter declared in lexically declared names",
                        position,
                    });
                }
            }
        }
        Ok(())
    }

    /// Check if the any of the parameters contains a yield expression.
    pub(crate) fn contains_yield_expression(&self) -> bool {
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
    pub(crate) fn contains_await_expression(&self) -> bool {
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
            parameters: parameters.into_boxed_slice(),
            flags,
            length,
        }
    }
}

impl From<FormalParameter> for FormalParameterList {
    fn from(parameter: FormalParameter) -> Self {
        let mut flags = FormalParameterListFlags::default();
        if parameter.is_rest_param() {
            flags |= FormalParameterListFlags::HAS_REST_PARAMETER;
        }
        if parameter.init().is_some() {
            flags |= FormalParameterListFlags::HAS_EXPRESSIONS;
        }
        if parameter.names().contains(&Sym::ARGUMENTS.into()) {
            flags |= FormalParameterListFlags::HAS_ARGUMENTS;
        }
        if parameter.is_rest_param() || parameter.init().is_some() || !parameter.is_identifier() {
            flags.remove(FormalParameterListFlags::IS_SIMPLE);
        }
        let length = if parameter.is_rest_param() || parameter.init().is_some() {
            0
        } else {
            1
        };
        Self {
            parameters: Box::new([parameter]),
            flags,
            length,
        }
    }
}

bitflags! {
    /// Flags for a [`FormalParameterList`].
    #[allow(clippy::unsafe_derive_deserialize)]
    #[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
    pub(crate) struct FormalParameterListFlags: u8 {
        const IS_SIMPLE = 0b0000_0001;
        const HAS_DUPLICATES = 0b0000_0010;
        const HAS_REST_PARAMETER = 0b0000_0100;
        const HAS_EXPRESSIONS = 0b0000_1000;
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
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct FormalParameter {
    variable: Variable,
    is_rest_param: bool,
}

impl FormalParameter {
    /// Creates a new formal parameter.
    pub(in crate::syntax) fn new<D>(variable: D, is_rest_param: bool) -> Self
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
