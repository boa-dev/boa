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

/// The kind of import in an [`ImportDeclaration`].
#[derive(Debug, Clone)]
pub enum ImportKind {
    /// Default (`import defaultName from "module-name"`) or null (`import "module-name").
    DefaultOrNull,
    /// Namespaced import (`import * as name from "module-name"`).
    Namespaced {
        /// Binding for the namespace created from the exports of the imported module.
        binding: Identifier,
    },
    /// Import list (`import { export1, export2 as alias2 } from "module-name"`).
    Named {
        /// List of the required exports of the imported module.
        names: Box<[ImportSpecifier]>,
    },
}

/// An import declaration AST node.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ImportDeclaration
#[derive(Debug, Clone)]
pub struct ImportDeclaration {
    /// Binding for the default export of `specifier`.
    default: Option<Identifier>,
    /// See [`ImportKind`].
    kind: ImportKind,
    /// Module specifier.
    specifier: ModuleSpecifier,
}

impl ImportDeclaration {
    /// Creates a new import declaration.
    #[inline]
    #[must_use]
    pub const fn new(
        default: Option<Identifier>,
        kind: ImportKind,
        specifier: ModuleSpecifier,
    ) -> Self {
        Self {
            default,
            kind,
            specifier,
        }
    }

    /// Gets the binding for the default export of the module.
    #[inline]
    #[must_use]
    pub const fn default(&self) -> Option<Identifier> {
        self.default
    }

    /// Gets the module specifier of the import declaration.
    #[inline]
    #[must_use]
    pub const fn specifier(&self) -> ModuleSpecifier {
        self.specifier
    }

    /// Gets the import kind of the import declaration
    #[inline]
    #[must_use]
    pub const fn kind(&self) -> &ImportKind {
        &self.kind
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
