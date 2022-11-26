//! Import declaration AST nodes.
//!
//! This module contains `import` declaration AST nodes.
//!
//! More information:
//! - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-imports
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import

use crate::expression::Identifier;
use boa_interner::Sym;

use super::FromClause;

/// An import declaration AST node.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportDeclaration
#[derive(Debug, Clone)]
pub enum ImportDeclaration {
    /// Full module import (`import "module-name"`).
    Module(Sym),
    /// Namespace import (`import * as name from "module-name"`), with an optional default export
    /// binding.
    Namespace {
        /// Optional default export for the namespace import.
        default_export: Option<Identifier>,
        /// Alias for the namespace import.
        alias: Identifier,
        /// From clause.
        from_clause: FromClause,
    },
    /// Import list (`import { export1, export2 as alias2} from "module-name"`), with an optional
    /// default export binding.
    List {
        /// Optional default export for the import list.
        default_export: Option<Identifier>,
        /// List of imports.
        import_list: Box<[ImportSpecifier]>,
        /// From clause.
        from_clause: FromClause,
    },
}

impl ImportDeclaration {
    /// Creates a new namespace import declaration.
    #[inline]
    pub fn namespace<F>(
        default_export: Option<Identifier>,
        alias: Identifier,
        from_clause: F,
    ) -> Self
    where
        F: Into<FromClause>,
    {
        Self::Namespace {
            default_export,
            alias,
            from_clause: from_clause.into(),
        }
    }

    /// Creates a new namespace import declaration.
    #[inline]
    pub fn list<L, F>(default_export: Option<Identifier>, import_list: L, from_clause: F) -> Self
    where
        L: Into<Box<[ImportSpecifier]>>,
        F: Into<FromClause>,
    {
        Self::List {
            default_export,
            import_list: import_list.into(),
            from_clause: from_clause.into(),
        }
    }
}

/// Import specifier
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportSpecifier
#[derive(Debug, Clone, Copy)]
pub struct ImportSpecifier {
    import_name: Sym,
    alias: Option<Identifier>,
}

impl ImportSpecifier {
    /// Creates a new [`ImportSpecifier`].
    #[inline]
    #[must_use]
    pub const fn new(import_name: Sym, alias: Option<Identifier>) -> Self {
        Self { import_name, alias }
    }

    /// Gets the original import name.
    #[inline]
    #[must_use]
    pub const fn import_name(self) -> Sym {
        self.import_name
    }

    /// Gets an optional import alias for the import.
    #[inline]
    #[must_use]
    pub const fn alias(self) -> Option<Identifier> {
        self.alias
    }
}
