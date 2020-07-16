use super::Node;
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Label {
    stmt: Box<Node>,
    label: Box<str>,
}

impl Label {
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{}: {}", self.label, self.stmt))
    }

    pub fn new(stmt: Node, label: Box<str>) -> Self {
        Self {
            stmt: stmt.into(),
            label,
        }
    }
}

impl From<Label> for Node {
    fn from(label_stmt: Label) -> Node {
        Node::Label(label_stmt)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f)
    }
}
