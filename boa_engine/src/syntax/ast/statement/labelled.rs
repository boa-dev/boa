use super::Statement;
use crate::syntax::ast::visitor::{VisitWith, Visitor, VisitorMut};
use crate::syntax::ast::{function::Function, ContainsSymbol};
use crate::try_break;
use boa_interner::{Interner, Sym, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

/// The set of Parse Nodes that can be preceded by a label, as defined by the [spec].
///
/// Semantically, a [`Labelled`] statement should only wrap [`Statement`] nodes. However,
/// old ECMAScript implementations supported [labelled function declarations][label-fn] as an extension
/// of the grammar. In the ECMAScript 2015 spec, the production of `LabelledStatement` was
/// modified to include labelled [`Function`]s as a valid node.
///
/// [spec]: https://tc39.es/ecma262/#prod-LabelledItem
/// [label-fn]: https://tc39.es/ecma262/#sec-labelled-function-declarations
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum LabelledItem {
    /// A labelled [`Function`].
    Function(Function),
    /// A labelled [`Statement`].
    Statement(Statement),
}

impl LabelledItem {
    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        match self {
            LabelledItem::Function(f) => f.to_indented_string(interner, indentation),
            LabelledItem::Statement(stmt) => stmt.to_indented_string(interner, indentation),
        }
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        match self {
            LabelledItem::Function(_) => false,
            LabelledItem::Statement(stmt) => stmt.contains_arguments(),
        }
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        match self {
            LabelledItem::Function(_) => false,
            LabelledItem::Statement(stmt) => stmt.contains(symbol),
        }
    }
}

impl ToInternedString for LabelledItem {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Function> for LabelledItem {
    fn from(f: Function) -> Self {
        Self::Function(f)
    }
}

impl From<Statement> for LabelledItem {
    fn from(stmt: Statement) -> Self {
        Self::Statement(stmt)
    }
}

impl VisitWith for LabelledItem {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            LabelledItem::Function(f) => visitor.visit_function(f),
            LabelledItem::Statement(s) => visitor.visit_statement(s),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            LabelledItem::Function(f) => visitor.visit_function_mut(f),
            LabelledItem::Statement(s) => visitor.visit_statement_mut(s),
        }
    }
}

/// Labelled statement nodes, as defined by the [spec].
///
/// The method [`Labelled::item`] doesn't return a [`Statement`] for compatibility reasons.
/// See [`LabelledItem`] for more information.
///
/// [spec]: https://tc39.es/ecma262/#sec-labelled-statements
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Labelled {
    item: Box<LabelledItem>,
    label: Sym,
}

impl Labelled {
    /// Creates a new `Labelled` statement.
    pub fn new(item: LabelledItem, label: Sym) -> Self {
        Self {
            item: Box::new(item),
            label,
        }
    }

    /// Gets the labelled item.
    pub fn item(&self) -> &LabelledItem {
        &self.item
    }

    /// Gets the label name.
    pub fn label(&self) -> Sym {
        self.label
    }

    pub(crate) fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "{}: {}",
            interner.resolve_expect(self.label),
            self.item.to_indented_string(interner, indentation)
        )
    }

    #[inline]
    pub(crate) fn contains_arguments(&self) -> bool {
        self.label == Sym::ARGUMENTS || self.item.contains_arguments()
    }

    #[inline]
    pub(crate) fn contains(&self, symbol: ContainsSymbol) -> bool {
        self.item.contains(symbol)
    }
}

impl ToInternedString for Labelled {
    fn to_interned_string(&self, interner: &Interner) -> String {
        self.to_indented_string(interner, 0)
    }
}

impl From<Labelled> for Statement {
    fn from(labelled: Labelled) -> Self {
        Self::Labelled(labelled)
    }
}

impl VisitWith for Labelled {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_labelled_item(&self.item));
        visitor.visit_sym(&self.label)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_labelled_item_mut(&mut self.item));
        visitor.visit_sym_mut(&mut self.label)
    }
}
