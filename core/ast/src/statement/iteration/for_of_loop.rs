use crate::operations::{ContainsSymbol, contains};
use crate::scope::Scope;
use crate::visitor::{VisitWith, Visitor, VisitorMut};
use crate::{
    expression::Expression,
    statement::{Statement, iteration::IterableLoopInitializer},
};
use boa_interner::{Interner, ToIndentedString, ToInternedString};
use core::ops::ControlFlow;

/// A `for...of` loop statement, as defined by the [spec].
///
/// [`for..of`][forof] statements loop over a sequence of values obtained from an iterable object (Array,
/// String, Map, generators).
///
/// This type combines `for..of` and [`for await...of`][forawait] statements in a single structure,
/// since `for await...of` is essentially the same statement but with async iterable objects
/// as the source of iteration.
///
/// [forof]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of
/// [spec]: https://tc39.es/ecma262/#prod-ForInOfStatement
/// [forawait]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for-await...of
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct ForOfLoop<'arena> {
    pub(crate) init: IterableLoopInitializer<'arena>,
    pub(crate) iterable: Expression<'arena>,
    pub(crate) body: Box<Statement<'arena>>,
    r#await: bool,
    pub(crate) iterable_contains_direct_eval: bool,
    pub(crate) contains_direct_eval: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) iterable_scope: Option<Scope>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) scope: Option<Scope>,
}

impl<'arena> ForOfLoop<'arena> {
    /// Creates a new "for of" loop AST node.
    #[inline]
    #[must_use]
    pub fn new(
        init: IterableLoopInitializer<'arena>,
        iterable: Expression<'arena>,
        body: Statement<'arena>,
        r#await: bool,
    ) -> Self {
        let iterable_contains_direct_eval = contains(&iterable, ContainsSymbol::DirectEval);
        let contains_direct_eval = contains(&init, ContainsSymbol::DirectEval)
            || contains(&body, ContainsSymbol::DirectEval);
        Self {
            init,
            iterable,
            body: body.into(),
            iterable_contains_direct_eval,
            contains_direct_eval,
            r#await,
            iterable_scope: None,
            scope: None,
        }
    }

    /// Gets the initializer of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn initializer(&self) -> &IterableLoopInitializer<'arena> {
        &self.init
    }

    /// Gets the iterable expression of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn iterable(&self) -> &Expression<'arena> {
        &self.iterable
    }

    /// Gets the body to execute in the for...of loop.
    #[inline]
    #[must_use]
    pub const fn body(&self) -> &Statement<'arena> {
        &self.body
    }

    /// Returns true if this "for...of" loop is an "for await...of" loop.
    #[inline]
    #[must_use]
    pub const fn r#await(&self) -> bool {
        self.r#await
    }

    /// Return the iterable scope of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn iterable_scope(&self) -> Option<&Scope> {
        self.iterable_scope.as_ref()
    }

    /// Return the scope of the for...of loop.
    #[inline]
    #[must_use]
    pub const fn scope(&self) -> Option<&Scope> {
        self.scope.as_ref()
    }
}

impl ToIndentedString for ForOfLoop<'_> {
    fn to_indented_string(&self, interner: &Interner, indentation: usize) -> String {
        format!(
            "for ({} of {}) {}",
            self.init.to_interned_string(interner),
            self.iterable.to_interned_string(interner),
            self.body().to_indented_string(interner, indentation)
        )
    }
}

impl<'arena> From<ForOfLoop<'arena>> for Statement<'arena> {
    #[inline]
    fn from(for_of: ForOfLoop<'arena>) -> Self {
        Self::ForOfLoop(for_of)
    }
}

impl<'arena> VisitWith<'arena> for ForOfLoop<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_iterable_loop_initializer(&self.init)?;
        visitor.visit_expression(&self.iterable)?;
        visitor.visit_statement(&self.body)
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_iterable_loop_initializer_mut(&mut self.init)?;
        visitor.visit_expression_mut(&mut self.iterable)?;
        visitor.visit_statement_mut(&mut self.body)
    }
}
