//! Template literal node.

use super::Node;
use boa_interner::{Interner, Sym, ToInternedString};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Template literals are string literals allowing embedded expressions.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
/// [spec]: https://tc39.es/ecma262/#sec-template-literals
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateLit {
    elements: Box<[TemplateElement]>,
}

impl TemplateLit {
    pub fn new<E>(elements: E) -> Self
    where
        E: Into<Box<[TemplateElement]>>,
    {
        Self {
            elements: elements.into(),
        }
    }

    pub(crate) fn elements(&self) -> &[TemplateElement] {
        &self.elements
    }
}

impl ToInternedString for TemplateLit {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = "`".to_owned();

        for elt in self.elements.iter() {
            match elt {
                TemplateElement::String(s) => buf.push_str(interner.resolve_expect(*s)),
                TemplateElement::Expr(n) => {
                    buf.push_str(&format!("${{{}}}", n.to_interned_string(interner)));
                }
            }
        }
        buf.push('`');

        buf
    }
}
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct TaggedTemplate {
    tag: Box<Node>,
    raws: Box<[Sym]>,
    cookeds: Box<[Option<Sym>]>,
    exprs: Box<[Node]>,
}

impl TaggedTemplate {
    /// Creates a new tagged template with a tag, the list of raw strings, the cooked strings and
    /// the expressions.
    pub fn new<R, C, E>(tag: Node, raws: R, cookeds: C, exprs: E) -> Self
    where
        R: Into<Box<[Sym]>>,
        C: Into<Box<[Option<Sym>]>>,
        E: Into<Box<[Node]>>,
    {
        Self {
            tag: Box::new(tag),
            raws: raws.into(),
            cookeds: cookeds.into(),
            exprs: exprs.into(),
        }
    }

    pub(crate) fn tag(&self) -> &Node {
        &self.tag
    }

    pub(crate) fn raws(&self) -> &[Sym] {
        &self.raws
    }

    pub(crate) fn cookeds(&self) -> &[Option<Sym>] {
        &self.cookeds
    }

    pub(crate) fn exprs(&self) -> &[Node] {
        &self.exprs
    }
}

impl ToInternedString for TaggedTemplate {
    fn to_interned_string(&self, interner: &Interner) -> String {
        let mut buf = format!("{}`", self.tag.to_interned_string(interner));
        for (&raw, expr) in self.raws.iter().zip(self.exprs.iter()) {
            buf.push_str(&format!(
                "{}${{{}}}",
                interner.resolve_expect(raw),
                expr.to_interned_string(interner)
            ));
        }
        buf.push('`');

        buf
    }
}

impl From<TaggedTemplate> for Node {
    fn from(template: TaggedTemplate) -> Self {
        Self::TaggedTemplate(Box::new(template))
    }
}

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum TemplateElement {
    String(Sym),
    Expr(Node),
}
