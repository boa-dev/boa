//! Block AST node.

use super::{Node, StatementList};
use crate::{
    environment::declarative_environment_record::DeclarativeEnvironmentRecord,
    exec::Executable,
    exec::InterpreterState,
    gc::{Finalize, Trace},
    BoaProfiler, Context, JsResult, JsValue,
};
use std::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// A `block` statement (or compound statement in other languages) is used to group zero or
/// more statements.
///
/// The block statement is often called compound statement in other languages.
/// It allows you to use multiple statements where JavaScript expects only one statement.
/// Combining statements into blocks is a common practice in JavaScript. The opposite behavior
/// is possible using an empty statement, where you provide no statement, although one is
/// required.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#prod-BlockStatement
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "deser", serde(transparent))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Block {
    #[cfg_attr(feature = "deser", serde(flatten))]
    statements: StatementList,
}

impl Block {
    /// Gets the list of statements and declarations in this block.
    pub(crate) fn items(&self) -> &[Node] {
        self.statements.items()
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        writeln!(f, "{{")?;
        self.statements.display(f, indentation + 1)?;
        write!(f, "{}}}", "    ".repeat(indentation))
    }
}

impl Executable for Block {
    fn run(&self, context: &mut Context) -> JsResult<JsValue> {
        let _timer = BoaProfiler::global().start_event("Block", "exec");
        {
            let env = context.get_current_environment();
            context.push_environment(DeclarativeEnvironmentRecord::new(Some(env)));
        }

        // https://tc39.es/ecma262/#sec-block-runtime-semantics-evaluation
        // The return value is uninitialized, which means it defaults to Value::Undefined
        let mut obj = JsValue::default();
        for statement in self.items() {
            obj = statement.run(context).map_err(|e| {
                // No matter how control leaves the Block the LexicalEnvironment is always
                // restored to its former state.
                context.pop_environment();
                e
            })?;

            match context.executor().get_current_state() {
                InterpreterState::Return => {
                    // Early return.
                    break;
                }
                InterpreterState::Break(_label) => {
                    // TODO, break to a label.

                    // Early break.
                    break;
                }
                InterpreterState::Continue(_label) => {
                    // TODO, continue to a label
                    break;
                }
                InterpreterState::Executing => {
                    // Continue execution
                }
            }
        }

        // pop the block env
        let _ = context.pop_environment();

        Ok(obj)
    }
}

impl<T> From<T> for Block
where
    T: Into<StatementList>,
{
    fn from(list: T) -> Self {
        Self {
            statements: list.into(),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display(f, 0)
    }
}

impl From<Block> for Node {
    fn from(block: Block) -> Self {
        Self::Block(block)
    }
}
