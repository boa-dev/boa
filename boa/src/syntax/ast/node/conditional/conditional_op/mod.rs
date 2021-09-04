use crate::{
    exec::Executable,
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    Context, JsResult, JsValue,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
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

impl Executable for ConditionalOp {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        Ok(if self.cond().run(context)?.to_boolean() {
            self.if_true().run(context)?
        } else {
            self.if_false().run(context)?
        })
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
