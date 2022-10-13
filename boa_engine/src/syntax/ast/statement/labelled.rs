//! Labelled statement.

use boa_interner::{Interner, Sym, ToInternedString};

use crate::syntax::ast::{function::Function, ContainsSymbol};

use super::Statement;

/// The set of AST nodes that can be preceded by a label, per the [spec].
///
/// [spec]: https://tc39.es/ecma262/#prod-LabelledItem
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum LabelledItem {
    Function(Function),
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

/// The labeled statement can be used with break or continue statements. It is prefixing a statement
/// with an identifier which you can refer to.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-labelled-statements
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/label
#[cfg_attr(feature = "deser", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Labelled {
    item: Box<LabelledItem>,
    label: Sym,
}

impl Labelled {
    pub fn new(item: LabelledItem, label: Sym) -> Self {
        Self {
            item: Box::new(item),
            label,
        }
    }

    pub fn item(&self) -> &LabelledItem {
        &self.item
    }

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
