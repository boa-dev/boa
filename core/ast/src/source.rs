use std::ops::ControlFlow;

use boa_interner::{Interner, Sym, ToIndentedString};

use crate::{
    ModuleItemList, StatementList,
    scope::Scope,
    scope_analyzer::{
        EvalDeclarationBindings, analyze_binding_escapes, collect_bindings,
        eval_declaration_instantiation_scope, optimize_scope_indices,
    },
    visitor::{VisitWith, Visitor, VisitorMut},
};

/// A Script source.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-scripts
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default)]
pub struct Script<'arena> {
    statements: StatementList<'arena>,
}

impl<'arena> Script<'arena> {
    /// Creates a new `ScriptNode`.
    #[must_use]
    pub const fn new(statements: StatementList<'arena>) -> Self {
        Self { statements }
    }

    /// Gets the list of statements of this `ScriptNode`.
    #[must_use]
    pub const fn statements(&self) -> &StatementList<'arena> {
        &self.statements
    }

    /// Gets a mutable reference to the list of statements of this `ScriptNode`.
    pub fn statements_mut(&mut self) -> &mut StatementList<'arena> {
        &mut self.statements
    }

    /// Gets the strict mode.
    #[inline]
    #[must_use]
    pub const fn strict(&self) -> bool {
        self.statements.strict()
    }

    /// Analyze the scope of the script.
    ///
    /// # Errors
    /// Any scope or binding errors that happened during the analysis.
    pub fn analyze_scope(
        &mut self,
        scope: &Scope,
        interner: &Interner,
    ) -> Result<(), &'static str> {
        collect_bindings(self, self.strict(), false, scope, interner)?;
        analyze_binding_escapes(self, false, scope.clone(), interner)?;
        optimize_scope_indices(self, scope);
        Ok(())
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
        annex_b_function_names: &[Sym],
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

        if let Err(reason) = collect_bindings(self, strict, true, lexical_scope, interner) {
            return Err(format!("Failed to analyze scope: {reason}"));
        }
        if let Err(reason) = analyze_binding_escapes(self, true, lexical_scope.clone(), interner) {
            return Err(format!("Failed to analyze scope: {reason}"));
        }

        variable_scope.escape_all_bindings();
        lexical_scope.escape_all_bindings();
        variable_scope.reorder_binding_indices();
        lexical_scope.reorder_binding_indices();
        optimize_scope_indices(self, lexical_scope);

        Ok(bindings)
    }
}

impl<'arena> VisitWith<'arena> for Script<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        self.statements.visit_with(visitor)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        self.statements.visit_with_mut(visitor)
    }
}

impl ToIndentedString for Script<'_> {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        self.statements.to_indented_string(interner, indentation)
    }
}

impl PartialEq for Script<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.statements == other.statements
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, 'arena> arbitrary::Arbitrary<'a> for Script<'arena> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let statements = StatementList::arbitrary(u)?;
        Ok(Self { statements })
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
pub struct Module<'arena> {
    pub(crate) items: ModuleItemList<'arena>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Scope,
}

impl<'arena> Module<'arena> {
    /// Creates a new `ModuleNode`.
    #[must_use]
    pub fn new(items: ModuleItemList<'arena>) -> Self {
        Self {
            items,
            scope: Scope::default(),
        }
    }

    /// Gets the list of items of this `ModuleNode`.
    #[must_use]
    pub const fn items(&self) -> &ModuleItemList<'arena> {
        &self.items
    }

    /// Gets the scope of this `ModuleNode`.
    #[inline]
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }

    /// Analyze the scope of the module.
    ///
    /// # Errors
    /// Any scope or binding errors that happened during the analysis.
    pub fn analyze_scope(
        &mut self,
        scope: &Scope,
        interner: &Interner,
    ) -> Result<(), &'static str> {
        collect_bindings(self, true, false, scope, interner)?;
        analyze_binding_escapes(self, false, scope.clone(), interner)?;
        optimize_scope_indices(self, &self.scope.clone());

        Ok(())
    }
}

impl<'arena> VisitWith<'arena> for Module<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        self.items.visit_with(visitor)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        self.items.visit_with_mut(visitor)
    }
}
