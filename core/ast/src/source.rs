use std::ops::ControlFlow;

use boa_interner::ToIndentedString;

use crate::{
    visitor::{VisitWith, Visitor, VisitorMut},
    ModuleItemList, StatementList,
};

/// A Script source.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-scripts
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Script {
    statements: StatementList,
}

impl Script {
    /// Creates a new `ScriptNode`.
    #[must_use]
    pub const fn new(statements: StatementList) -> Self {
        Self { statements }
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
    fn to_indented_string(&self, interner: &boa_interner::Interner, indentation: usize) -> String {
        self.statements.to_indented_string(interner, indentation)
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
    items: ModuleItemList,
}

impl Module {
    /// Creates a new `ModuleNode`.
    #[must_use]
    pub const fn new(items: ModuleItemList) -> Self {
        Self { items }
    }

    /// Gets the list of itemos of this `ModuleNode`.
    #[must_use]
    pub const fn items(&self) -> &ModuleItemList {
        &self.items
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
