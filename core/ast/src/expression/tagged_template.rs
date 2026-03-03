use super::Expression;
use crate::{
    Span, Spanned,
    visitor::{VisitWith, Visitor, VisitorMut},
};
use boa_interner::{Interner, Sym, ToInternedString};
use core::{fmt::Write as _, ops::ControlFlow};

/// A [`TaggedTemplate`][moz] expression, as defined by the [spec].
///
/// `TaggedTemplate`s are a type of template literals that are parsed by a custom function to generate
/// arbitrary objects from the inner strings and expressions.
///
/// [moz]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals#tagged_templates
/// [spec]: https://tc39.es/ecma262/#sec-tagged-templates
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct TaggedTemplate<'arena> {
    tag: Box<Expression<'arena>>,
    raws: Box<[Sym]>,
    cookeds: Box<[Option<Sym>]>,
    exprs: Box<[Expression<'arena>]>,
    identifier: u64,
    span: Span,
}

impl<'arena> TaggedTemplate<'arena> {
    /// Creates a new tagged template with a tag, the list of raw strings, the cooked strings and
    /// the expressions.
    #[inline]
    #[must_use]
    pub fn new(
        tag: Expression<'arena>,
        raws: Box<[Sym]>,
        cookeds: Box<[Option<Sym>]>,
        exprs: Box<[Expression<'arena>]>,
        identifier: u64,
        span: Span,
    ) -> Self {
        Self {
            tag: tag.into(),
            raws,
            cookeds,
            exprs,
            identifier,
            span,
        }
    }

    /// Gets the tag function of the template.
    #[inline]
    #[must_use]
    pub const fn tag(&self) -> &Expression<'arena> {
        &self.tag
    }

    /// Gets the inner raw strings of the template.
    #[inline]
    #[must_use]
    pub const fn raws(&self) -> &[Sym] {
        &self.raws
    }

    /// Gets the cooked strings of the template.
    #[inline]
    #[must_use]
    pub const fn cookeds(&self) -> &[Option<Sym>] {
        &self.cookeds
    }

    /// Gets the interpolated expressions of the template.
    #[inline]
    #[must_use]
    pub const fn exprs(&self) -> &[Expression<'arena>] {
        &self.exprs
    }

    /// Gets the unique identifier of the template.
    #[inline]
    #[must_use]
    pub const fn identifier(&self) -> u64 {
        self.identifier
    }
}

impl<'arena> Spanned for TaggedTemplate<'arena> {
    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena> ToInternedString for TaggedTemplate<'arena> {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = format!("{}`", self.tag.to_interned_string(interner));
        let mut exprs = self.exprs.iter();

        for raw in &self.raws {
            let _ = write!(buf, "{}", interner.resolve_expect(*raw));
            if let Some(expr) = exprs.next() {
                let _ = write!(buf, "${{{}}}", expr.to_interned_string(interner));
            }
        }
        buf.push('`');

        buf
    }
}

impl<'arena> From<TaggedTemplate<'arena>> for Expression<'arena> {
    #[inline]
    fn from(template: TaggedTemplate<'arena>) -> Self {
        Self::TaggedTemplate(template)
    }
}

impl<'arena> VisitWith<'arena> for TaggedTemplate<'arena> {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a, 'arena>,
    {
        visitor.visit_expression(&self.tag)?;
        for raw in &*self.raws {
            visitor.visit_sym(raw)?;
        }
        for cooked in self.cookeds.iter().flatten() {
            visitor.visit_sym(cooked)?;
        }
        for expr in &*self.exprs {
            visitor.visit_expression(expr)?;
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a, 'arena>,
    {
        visitor.visit_expression_mut(&mut self.tag)?;
        for raw in &mut *self.raws {
            visitor.visit_sym_mut(raw)?;
        }
        for cooked in self.cookeds.iter_mut().flatten() {
            visitor.visit_sym_mut(cooked)?;
        }
        for expr in &mut *self.exprs {
            visitor.visit_expression_mut(expr)?;
        }
        ControlFlow::Continue(())
    }
}
