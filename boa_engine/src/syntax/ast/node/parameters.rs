use super::{Declaration, DeclarationPattern, Node};
use bitflags::bitflags;
use boa_gc::{Finalize, Trace};
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// `FormalParameterList` is a list of `FormalParameter`s that describes the parameters of a function.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameterList
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default, PartialEq, Trace, Finalize)]
pub struct FormalParameterList {
    pub(crate) parameters: Box<[FormalParameter]>,
    #[unsafe_ignore_trace]
    pub(crate) flags: FormalParameterListFlags,
}

impl FormalParameterList {
    /// Creates a new formal parameter list.
    pub(crate) fn new(parameters: Box<[FormalParameter]>, flags: FormalParameterListFlags) -> Self {
        Self { parameters, flags }
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
}

bitflags! {
    /// Flags for a [`FormalParameterList`].
    #[allow(clippy::unsafe_derive_deserialize)]
    #[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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
///```text
///function foo(formalParameter1, formalParameter2) {
///}
///```
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-FormalParameter
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Errors/Missing_formal_parameter
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct FormalParameter {
    declaration: Declaration,
    is_rest_param: bool,
}

impl FormalParameter {
    /// Creates a new formal parameter.
    pub(in crate::syntax) fn new<D>(declaration: D, is_rest_param: bool) -> Self
    where
        D: Into<Declaration>,
    {
        Self {
            declaration: declaration.into(),
            is_rest_param,
        }
    }

    /// Gets the name of the formal parameter.
    pub fn names(&self) -> Vec<Sym> {
        match &self.declaration {
            Declaration::Identifier { ident, .. } => vec![ident.sym()],
            Declaration::Pattern(pattern) => match pattern {
                DeclarationPattern::Object(object_pattern) => object_pattern.idents(),

                DeclarationPattern::Array(array_pattern) => array_pattern.idents(),
            },
        }
    }

    /// Get the declaration of the formal parameter
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }

    /// Gets the initialization node of the formal parameter, if any.
    pub fn init(&self) -> Option<&Node> {
        self.declaration.init()
    }

    /// Gets wether the parameter is a rest parameter.
    pub fn is_rest_param(&self) -> bool {
        self.is_rest_param
    }

    pub fn is_identifier(&self) -> bool {
        matches!(&self.declaration, Declaration::Identifier { .. })
    }
}

impl ToInternedString for FormalParameter {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = if self.is_rest_param {
            "...".to_owned()
        } else {
            String::new()
        };
        buf.push_str(&self.declaration.to_interned_string(interner));
        buf
    }
}
