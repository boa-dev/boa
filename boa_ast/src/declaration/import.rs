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

use super::ModuleSpecifier;

/// An import declaration AST node.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportDeclaration
#[derive(Debug, Clone)]
pub enum ImportDeclaration {
    /// Full module import (`import "module-name"`).
    Module(ModuleSpecifier),
    /// Namespace import (`import * as name from "module-name"`), with an optional default export
    /// binding.
    Namespace {
        /// Optional default export for the namespace import.
        default_export: Option<Identifier>,
        /// Alias for the namespace import.
        alias: Identifier,
        /// Module specifier.
        specifier: ModuleSpecifier,
    },
    /// Import list (`import { export1, export2 as alias2} from "module-name"`), with an optional
    /// default export binding.
    List {
        /// Optional default export for the import list.
        default_export: Option<Identifier>,
        /// List of imports.
        import_list: Box<[ImportSpecifier]>,
        /// Module specifier.
        specifier: ModuleSpecifier,
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
        F: Into<ModuleSpecifier>,
    {
        Self::Namespace {
            default_export,
            alias,
            specifier: from_clause.into(),
        }
    }

    /// Creates a new namespace import declaration.
    #[inline]
    pub fn list<L, F>(default_export: Option<Identifier>, import_list: L, from_clause: F) -> Self
    where
        L: Into<Box<[ImportSpecifier]>>,
        F: Into<ModuleSpecifier>,
    {
        Self::List {
            default_export,
            import_list: import_list.into(),
            specifier: from_clause.into(),
        }
    }

    /// Gets the module specifier of the import.
    #[must_use]
    pub const fn module_specifier(&self) -> ModuleSpecifier {
        match self {
            Self::Module(specifier)
            | Self::Namespace { specifier, .. }
            | Self::List { specifier, .. } => *specifier,
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
    binding: Identifier,
    export_name: Sym,
}

impl ImportSpecifier {
    /// Creates a new [`ImportSpecifier`].
    #[inline]
    #[must_use]
    pub const fn new(binding: Identifier, export_name: Sym) -> Self {
        Self {
            binding,
            export_name,
        }
    }

    /// Gets the binding of the import specifier.
    #[inline]
    #[must_use]
    pub const fn binding(self) -> Identifier {
        self.binding
    }

    /// Gets the optional export name of the import.
    #[inline]
    #[must_use]
    pub const fn export_name(self) -> Sym {
        self.export_name
    }
}
