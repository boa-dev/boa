//! Template literal node.

use super::Node;
use gc::{Finalize, Trace};

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};
use std::fmt;

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
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct TemplateLit {
    elements: Vec<TemplateElement>,
}

impl TemplateLit {
    pub fn new(elements: Vec<TemplateElement>) -> Self {
        TemplateLit { elements }
    }

    pub(crate) fn elements(&self) -> &Vec<TemplateElement> {
        &self.elements
    }
}

impl fmt::Display for TemplateLit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`")?;
        for elt in &self.elements {
            match elt {
                TemplateElement::String(s) => write!(f, "{}", s)?,
                TemplateElement::Expr(n) => write!(f, "${{{}}}", n)?,
            }
        }
        write!(f, "`")
    }
}
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct TaggedTemplate {
    tag: Box<Node>,
    raws: Vec<Box<str>>,
    cookeds: Vec<Option<Box<str>>>,
    exprs: Vec<Node>,
}

impl TaggedTemplate {
    pub fn new(
        tag: Node,
        raws: Vec<Box<str>>,
        cookeds: Vec<Option<Box<str>>>,
        exprs: Vec<Node>,
    ) -> Self {
        Self {
            tag: Box::new(tag),
            raws,
            cookeds,
            exprs,
        }
    }

    pub(crate) fn tag(&self) -> &Node {
        &self.tag
    }

    pub(crate) fn raws(&self) -> &Vec<Box<str>> {
        &self.raws
    }

    pub(crate) fn cookeds(&self) -> &Vec<Option<Box<str>>> {
        &self.cookeds
    }

    pub(crate) fn exprs(&self) -> &Vec<Node> {
        &self.exprs
    }
}

impl fmt::Display for TaggedTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}`", self.tag)?;
        for (raw, expr) in self.raws.iter().zip(self.exprs.iter()) {
            write!(f, "{}${{{}}}", raw, expr)?;
        }
        write!(f, "`")
    }
}

impl From<TaggedTemplate> for Node {
    fn from(template: TaggedTemplate) -> Self {
        Node::TaggedTemplate(template)
    }
}

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub enum TemplateElement {
    String(Box<str>),
    Expr(Node),
}
