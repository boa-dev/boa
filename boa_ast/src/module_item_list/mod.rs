//! Module item list AST nodes.
//!
//! More information:
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-modules

use crate::{
    declaration::{ExportDeclaration, ImportDeclaration},
    StatementListItem,
};

/// Module item list AST node.
///
/// It contains a list of
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ModuleItemList
#[derive(Debug, Clone)]
pub struct ModuleItemList {
    items: Box<[ModuleItem]>,
}

impl ModuleItemList {
    /// Gets the list of module items.
    #[inline]
    #[must_use]
    pub const fn items(&self) -> &[ModuleItem] {
        &self.items
    }
}

impl<T> From<T> for ModuleItemList
where
    T: Into<Box<[ModuleItem]>>,
{
    #[inline]
    fn from(items: T) -> Self {
        Self {
            items: items.into(),
        }
    }
}

/// Module item AST node.
///
/// This is an extension over a [`StatementList`], which can also include multiple
/// [`ImportDeclaration`] and [`ExportDeclaration`] nodes.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ModuleItem
#[derive(Debug, Clone)]
pub enum ModuleItem {
    /// See [`ImportDeclaration`].
    ImportDeclaration(ImportDeclaration),
    /// See [`ExportDeclaration`].
    ExportDeclaration(ExportDeclaration),
    /// See [`StatementListItem`].
    StatementListItem(StatementListItem),
}
