//! Template literal Expression.

use core::ops::ControlFlow;
use std::borrow::Cow;

use boa_interner::{Interner, Sym, ToInternedString};

use crate::{
    expression::Expression,
    try_break,
    visitor::{VisitWith, Visitor, VisitorMut},
    ToStringEscaped,
};

/// Template literals are string literals allowing embedded expressions.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
/// [spec]: https://tc39.es/ecma262/#sec-template-literals
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateLiteral {
    elements: Box<[TemplateElement]>,
}

impl From<TemplateLiteral> for Expression {
    #[inline]
    fn from(tem: TemplateLiteral) -> Self {
        Self::TemplateLiteral(tem)
    }
}

/// An element found within a [`TemplateLiteral`].
///
/// The [spec] doesn't define an element akin to `TemplateElement`. However, the AST defines this
/// node as the equivalent of the components found in a template literal.
///
/// [spec]: https://tc39.es/ecma262/#sec-template-literals
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[derive(Clone, Debug, PartialEq)]
pub enum TemplateElement {
    /// A simple string.
    String(Sym),
    /// An expression that is evaluated and replaced by its string representation.
    Expr(Expression),
}

impl TemplateLiteral {
    /// Creates a new `TemplateLiteral` from a list of [`TemplateElement`]s.
    #[inline]
    #[must_use]
    pub fn new(elements: Box<[TemplateElement]>) -> Self {
        Self { elements }
    }

    /// Gets the element list of this `TemplateLiteral`.
    #[must_use]
    pub const fn elements(&self) -> &[TemplateElement] {
        &self.elements
    }
}

impl ToInternedString for TemplateLiteral {
    #[inline]
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "`".to_owned();

        for elt in self.elements.iter() {
            match elt {
                TemplateElement::String(s) => buf.push_str(&interner.resolve_expect(*s).join(
                    Cow::Borrowed,
                    |utf16| Cow::Owned(utf16.to_string_escaped()),
                    true,
                )),
                TemplateElement::Expr(n) => {
                    buf.push_str(&format!("${{{}}}", n.to_interned_string(interner)));
                }
            }
        }
        buf.push('`');

        buf
    }
}

impl VisitWith for TemplateLiteral {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        for element in self.elements.iter() {
            try_break!(visitor.visit_template_element(element));
        }
        ControlFlow::Continue(())
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        for element in self.elements.iter_mut() {
            try_break!(visitor.visit_template_element_mut(element));
        }
        ControlFlow::Continue(())
    }
}

impl VisitWith for TemplateElement {
    fn visit_with<'a, V>(&'a self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'a>,
    {
        match self {
            Self::String(sym) => visitor.visit_sym(sym),
            Self::Expr(expr) => visitor.visit_expression(expr),
        }
    }

    fn visit_with_mut<'a, V>(&'a mut self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: VisitorMut<'a>,
    {
        match self {
            Self::String(sym) => visitor.visit_sym_mut(sym),
            Self::Expr(expr) => visitor.visit_expression_mut(expr),
        }
    }
}
