//! This module contains the ECMAScript representation regular expressions.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-regexp-regular-expression-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions

use std::ops::ControlFlow;

use boa_interner::{Interner, Sym, ToInternedString};

use crate::{
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
};

use super::Expression;

/// Regular expressions in ECMAScript.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-regexp-regular-expression-objects
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegExp {
    pattern: Sym,
    flags: Sym,
}

impl RegExp {
    /// Create a new [`RegExp`].
    #[inline]
    #[must_use]
    pub const fn new(pattern: Sym, flags: Sym) -> Self {
        Self { pattern, flags }
    }

    /// Get the pattern part of the [`RegExp`].
    #[inline]
    #[must_use]
    pub const fn pattern(&self) -> Sym {
        self.pattern
    }

    /// Get the flags part of the [`RegExp`].
    #[inline]
    #[must_use]
    pub const fn flags(&self) -> Sym {
        self.flags
    }
}

impl ToInternedString for RegExp {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let pattern = interner.resolve_expect(self.pattern);
        let flags = interner.resolve_expect(self.flags);
        format!("/{pattern}/{flags}")
    }
}

impl From<RegExp> for Expression {
    #[inline]
    fn from(value: RegExp) -> Self {
        Self::RegExp(value)
    }
}

impl VisitWith for RegExp {
    #[inline]
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        try_break!(visitor.visit_sym(&self.pattern));
        visitor.visit_sym(&self.flags)
    }

    #[inline]
    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        try_break!(visitor.visit_sym_mut(&mut self.pattern));
        visitor.visit_sym_mut(&mut self.flags)
    }
}
