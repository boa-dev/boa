use std::ops::ControlFlow;

use boa_interner::{Interner, ToIndentedString};

use crate::{
    expression::Identifier,
    scope::Scope,
    scope_analyzer::{
        analyze_binding_escapes, collect_bindings, eval_declaration_instantiation_scope,
        EvalDeclarationBindings,
    },
    visitor::{VisitWith, Visitor, VisitorMut},
    ModuleItemList, SourceText, StatementList,
};

/// A Script source.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-scripts
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct Script {
    statements: StatementList,
    source: Option<SourceText>,
}

impl Script {
    /// Creates a new `ScriptNode`.
    #[must_use]
    pub const fn new(statements: StatementList, source: Option<SourceText>) -> Self {
        Self { statements, source }
    }

    /// Gets the list of statements of this `ScriptNode`.
    #[must_use]
    pub const fn statements(&self) -> &StatementList {
        &self.statements
    }

    /// Gets a mutable reference to the list of statements of this `ScriptNode`.
    pub fn statements_mut(&mut self) -> &mut StatementList {
        &mut self.statements
    }

    /// Gets the strict mode.
    #[inline]
    #[must_use]
    pub const fn strict(&self) -> bool {
        self.statements.strict()
    }

    /// Analyze the scope of the script.
    pub fn analyze_scope(&mut self, scope: &Scope, interner: &Interner) -> bool {
        if !collect_bindings(self, self.strict(), false, scope, interner) {
            return false;
        }
        analyze_binding_escapes(self, false, scope.clone(), interner)
    }

    /// Analyze the scope of the script in eval mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the scope analysis fails with a syntax error.
    pub fn analyze_scope_eval(
        &mut self,
        strict: bool,
        variable_scope: &Scope,
        lexical_scope: &Scope,
        annex_b_function_names: &[Identifier],
        interner: &Interner,
    ) -> Result<EvalDeclarationBindings, String> {
        let bindings = eval_declaration_instantiation_scope(
            self,
            strict,
            variable_scope,
            lexical_scope,
            annex_b_function_names,
            interner,
        )?;

        if !collect_bindings(self, strict, true, lexical_scope, interner) {
            return Err(String::from("Failed to analyze scope"));
        }

        if !analyze_binding_escapes(self, true, lexical_scope.clone(), interner) {
            return Err(String::from("Failed to analyze scope"));
        }

        Ok(bindings)
    }

    /// Takes the source text.
    pub fn take_source(&mut self) -> Option<SourceText> {
        self.source.take()
    }
}

impl VisitWith for Script {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        self.statements.visit_with(visitor)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        self.statements.visit_with_mut(visitor)
    }
}

impl ToIndentedString for Script {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        self.statements.to_indented_string(interner, indentation)
    }
}

impl PartialEq for Script {
    fn eq(&self, other: &Self) -> bool {
        self.statements == other.statements
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Script {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let statements = StatementList::arbitrary(u)?;
        Ok(Self {
            statements,
            source: None,
        })
    }
}

/// A Module source.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-modules
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Module {
    pub(crate) items: ModuleItemList,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Scope,
}

impl Module {
    /// Creates a new `ModuleNode`.
    #[must_use]
    pub fn new(items: ModuleItemList) -> Self {
        Self {
            items,
            scope: Scope::default(),
        }
    }

    /// Gets the list of itemos of this `ModuleNode`.
    #[must_use]
    pub const fn items(&self) -> &ModuleItemList {
        &self.items
    }

    /// Gets the scope of this `ModuleNode`.
    #[inline]
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }

    /// Analyze the scope of the module.
    pub fn analyze_scope(&mut self, scope: &Scope, interner: &Interner) -> bool {
        if !collect_bindings(self, true, false, scope, interner) {
            return false;
        }
        analyze_binding_escapes(self, false, scope.clone(), interner)
    }
}

impl VisitWith for Module {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        self.items.visit_with(visitor)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        self.items.visit_with_mut(visitor)
    }
}
