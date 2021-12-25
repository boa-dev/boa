use crate::{
    gc::{Finalize, Trace},
    syntax::ast::node::{iteration::IterableLoopInitializer, Node},
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ForOfLoop {
    init: Box<IterableLoopInitializer>,
    iterable: Box<Node>,
    body: Box<Node>,
    label: Option<Box<str>>,
}

impl ForOfLoop {
    pub fn new<I, B>(init: IterableLoopInitializer, iterable: I, body: B) -> Self
    where
        I: Into<Node>,
        B: Into<Node>,
    {
        Self {
            init: Box::new(init),
            iterable: Box::new(iterable.into()),
            body: Box::new(body.into()),
            label: None,
        }
    }

    pub fn init(&self) -> &IterableLoopInitializer {
        &self.init
    }

    pub fn iterable(&self) -> &Node {
        &self.iterable
    }

    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_ref().map(Box::as_ref)
    }

    pub fn set_label(&mut self, label: Box<str>) {
        self.label = Some(label);
    }

    pub fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        if let Some(ref label) = self.label {
            write!(f, "{}: ", label)?;
        }
        write!(f, "for ({} of {}) ", self.init, self.iterable)?;
        self.body().display(f, indentation)
    }
}

impl fmt::Display for ForOfLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<ForOfLoop> for Node {
    fn from(for_of: ForOfLoop) -> Node {
        Self::ForOfLoop(for_of)
    }
}
