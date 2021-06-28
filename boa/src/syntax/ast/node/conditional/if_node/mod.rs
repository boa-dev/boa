use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::{Node, NodeKind},
    Context, Result, Value,
};
use std::fmt;

#[cfg(feature = "deser")]
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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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

    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
    ) -> fmt::Result {
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

impl Executable for If {
    fn run(&self, context: &mut Context) -> Result<Value> {
        Ok(if self.cond().run(context)?.to_boolean() {
            self.body().run(context)?
        } else if let Some(ref else_e) = self.else_node() {
            else_e.run(context)?
        } else {
            Value::undefined()
        })
    }
}

impl fmt::Display for If {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<If> for NodeKind {
    fn from(if_stm: If) -> Self {
        Self::If(if_stm)
    }
}
