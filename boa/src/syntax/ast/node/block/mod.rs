//! Block AST node.

use super::{Node, StatementList};
use crate::{
    environment::lexical_environment::new_declarative_environment, exec::Executable,
    exec::InterpreterState, BoaProfiler, Context, Result, Value,
};
use gc::{Finalize, Trace};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Clone, Debug, Trace, Finalize, PartialEq)]
pub struct Block {
    #[cfg_attr(feature = "serde", serde(flatten))]
    statements: StatementList,
}

impl Block {
    /// Gets the list of statements in this block.
    pub(crate) fn statements(&self) -> &[Node] {
        self.statements.statements()
    }

    /// Implements the display formatting with indentation.
    pub(super) fn display(&self, f: &mut fmt::Formatter<'_>, indentation: usize) -> fmt::Result {
        writeln!(f, "{{")?;
        self.statements.display(f, indentation + 1)?;
        write!(f, "{}}}", "    ".repeat(indentation))
    }
}

impl Executable for Block {
    fn run(&self, interpreter: &mut Context) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("Block", "exec");
        {
            let env = &mut interpreter.realm_mut().environment;
            env.push(new_declarative_environment(Some(
                env.get_current_environment_ref().clone(),
            )));
        }

        // https://tc39.es/ecma262/#sec-block-runtime-semantics-evaluation
        // The return value is uninitialized, which means it defaults to Value::Undefined
        let mut obj = Value::default();
        for statement in self.statements() {
            obj = statement.run(interpreter)?;

            match interpreter.executor().get_current_state() {
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
        let _ = interpreter.realm_mut().environment.pop();

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
