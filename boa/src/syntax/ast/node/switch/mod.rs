//! Switch node.
//!
use crate::{
    exec::{Executable, InterpreterState},
    gc::{Finalize, Trace},
    syntax::ast::node::Node,
    Context, JsResult, JsValue,
};
use std::fmt;

use crate::syntax::ast::node::StatementList;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Case {
    condition: Node,
    body: StatementList,
}

impl Case {
    /// Creates a `Case` AST node.
    pub fn new<C, B>(condition: C, body: B) -> Self
    where
        C: Into<Node>,
        B: Into<StatementList>,
    {
        Self {
            condition: condition.into(),
            body: body.into(),
        }
    }

    /// Gets the condition of the case.
    pub fn condition(&self) -> &Node {
        &self.condition
    }

    /// Gets the statement listin the body of the case.
    pub fn body(&self) -> &StatementList {
        &self.body
    }
}

/// The `switch` statement evaluates an expression, matching the expression's value to a case
/// clause, and executes statements associated with that case, as well as statements in cases
/// that follow the matching case.
///
/// A `switch` statement first evaluates its expression. It then looks for the first case
/// clause whose expression evaluates to the same value as the result of the input expression
/// (using the strict comparison, `===`) and transfers control to that clause, executing the
/// associated statements. (If multiple cases match the provided value, the first case that
/// matches is selected, even if the cases are not equal to each other.)
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-SwitchStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Switch {
    val: Box<Node>,
    cases: Box<[Case]>,
    default: Option<StatementList>,
}

impl Switch {
    /// Creates a `Switch` AST node.
    pub fn new<V, C, D>(val: V, cases: C, default: Option<D>) -> Self
    where
        V: Into<Node>,
        C: Into<Box<[Case]>>,
        D: Into<StatementList>,
    {
        Self {
            val: Box::new(val.into()),
            cases: cases.into(),
            default: default.map(D::into),
        }
    }

    /// Gets the value to switch.
    pub fn val(&self) -> &Node {
        &self.val
    }

    /// Gets the list of cases for the switch statement.
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }

    /// Gets the default statement list, if any.
    pub fn default(&self) -> Option<&[Node]> {
        self.default.as_ref().map(StatementList::items)
    }

    /// Implements the display formatting with indentation.
    pub(in crate::syntax::ast::node) fn display(
        &self,
        f: &mut fmt::Formatter<'_>,
        indentation: usize,
    ) -> fmt::Result {
        let indent = "    ".repeat(indentation);
        writeln!(f, "switch ({}) {{", self.val())?;
        for e in self.cases().iter() {
            writeln!(f, "{}    case {}:", indent, e.condition())?;
            e.body().display(f, indentation + 2)?;
        }

        if let Some(ref default) = self.default {
            writeln!(f, "{}    default:", indent)?;
            default.display(f, indentation + 2)?;
        }
        write!(f, "{}}}", indent)
    }
}

impl Executable for Switch {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let val = self.val().run(context)?;
        let mut result = JsValue::null();
        let mut matched = false;
        context
            .executor()
            .set_current_state(InterpreterState::Executing);

        // If a case block does not end with a break statement then subsequent cases will be run without
        // checking their conditions until a break is encountered.
        let mut fall_through: bool = false;

        for case in self.cases().iter() {
            let cond = case.condition();
            let block = case.body();
            if fall_through || val.strict_equals(&cond.run(context)?) {
                matched = true;
                let result = block.run(context)?;
                match context.executor().get_current_state() {
                    InterpreterState::Return => {
                        // Early return.
                        return Ok(result);
                    }
                    InterpreterState::Break(_label) => {
                        // TODO, break to a label.
                        // Break statement encountered so therefore end switch statement.
                        context
                            .executor()
                            .set_current_state(InterpreterState::Executing);
                        break;
                    }
                    InterpreterState::Continue(_label) => {
                        // TODO, continue to a label.
                        break;
                    }
                    InterpreterState::Executing => {
                        // Continuing execution / falling through to next case statement(s).
                        fall_through = true;
                    }
                }
            }
        }

        if !matched {
            if let Some(default) = self.default() {
                context
                    .executor()
                    .set_current_state(InterpreterState::Executing);
                for (i, item) in default.iter().enumerate() {
                    let val = item.run(context)?;
                    match context.executor().get_current_state() {
                        InterpreterState::Return => {
                            // Early return.
                            result = val;
                            break;
                        }
                        InterpreterState::Break(_label) => {
                            // TODO, break to a label.

                            // Early break.
                            break;
                        }
                        _ => {
                            // Continue execution
                        }
                    }
                    if i == default.len() - 1 {
                        result = val;
                    }
                }
            }
        }

        Ok(result)
    }
}

impl fmt::Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<Switch> for Node {
    fn from(switch: Switch) -> Self {
        Self::Switch(switch)
    }
}
