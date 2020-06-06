use super::Node;
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The `if` statement executes a statement if a specified condition is [`truthy`][truthy]. If
/// the condition is [`falsy`][falsy], another statement can be executed.
///
/// Multiple `if...else` statements can be nested to create an else if clause.
///
/// Note that there is no elseif (in one word) keyword in JavaScript.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-IfStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
/// [truthy]: https://developer.mozilla.org/en-US/docs/Glossary/truthy
/// [falsy]: https://developer.mozilla.org/en-US/docs/Glossary/falsy
/// [expression]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct If {
    cond: Box<Node>,
    body: Box<Node>,
    else_node: Option<Box<Node>>,
}

impl If {
    pub fn cond(&self) -> &Node {
        &self.cond
    }

    pub fn body(&self) -> &Node {
        &self.body
    }

    pub fn else_node(&self) -> Option<&Node> {
        self.else_node.as_ref().map(Box::as_ref)
    }

    /// Creates an `If` AST node.
    pub fn new<C, B, E, OE>(condition: C, body: B, else_node: OE) -> Self
    where
        C: Into<Node>,
        B: Into<Node>,
        E: Into<Node>,
        OE: Into<Option<E>>,
    {
        Self {
            cond: Box::new(condition.into()),
            body: Box::new(body.into()),
            else_node: else_node.into().map(E::into).map(Box::new),
        }
    }

    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        write!(f, "if ({}) ", self.cond())?;
        match self.else_node() {
            Some(else_e) => {
                self.body().display(f, indent)?;
                f.write_str(" else ")?;
                else_e.display(f, indent)
            }
            None => self.body().display(f, indent),
        }
    }
}

impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<If> for Node {
    fn from(if_stm: If) -> Node {
        Self::If(if_stm)
    }
}

/// The `conditional` (ternary) operator is the only JavaScript operator that takes three
/// operands.
///
/// This operator is the only JavaScript operator that takes three operands: a condition
/// followed by a question mark (`?`), then an expression to execute `if` the condition is
/// truthy followed by a colon (`:`), and finally the expression to execute if the condition
/// is `false`. This operator is frequently used as a shortcut for the `if` statement.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-ConditionalExpression
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#Literals
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct ConditionalOp {
    condition: Box<Node>,
    if_true: Box<Node>,
    if_false: Box<Node>,
}

impl ConditionalOp {
    pub fn cond(&self) -> &Node {
        &self.condition
    }

    pub fn if_true(&self) -> &Node {
        &self.if_true
    }

    pub fn if_false(&self) -> &Node {
        &self.if_false
    }

    /// Creates a `ConditionalOp` AST node.
    pub fn new<C, T, F>(condition: C, if_true: T, if_false: F) -> Self
    where
        C: Into<Node>,
        T: Into<Node>,
        F: Into<Node>,
    {
        Self {
            condition: Box::new(condition.into()),
            if_true: Box::new(if_true.into()),
            if_false: Box::new(if_false.into()),
        }
    }
}

impl fmt::Display for ConditionalOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ? {} : {}",
            self.cond(),
            self.if_true(),
            self.if_false()
        )
    }
}

impl From<ConditionalOp> for Node {
    fn from(cond_op: ConditionalOp) -> Node {
        Self::ConditionalOp(cond_op)
    }
}
