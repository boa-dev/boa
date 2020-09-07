//! Statement list execution.

use super::{Executable, Interpreter, InterpreterState};
use crate::{syntax::ast::node::StatementList, BoaProfiler, Result, Value};

impl Executable for StatementList {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let _timer = BoaProfiler::global().start_event("StatementList", "exec");

        // https://tc39.es/ecma262/#sec-block-runtime-semantics-evaluation
        // The return value is uninitialized, which means it defaults to Value::Undefined
        let mut obj = Value::default();
        interpreter.set_current_state(InterpreterState::Executing);
        for (i, item) in self.statements().iter().enumerate() {
            let val = item.run(interpreter)?;
            match interpreter.get_current_state() {
                InterpreterState::Return => {
                    // Early return.
                    obj = val;
                    break;
                }
                InterpreterState::Break(_label) => {
                    // TODO, break to a label.

                    // Early break.
                    break;
                }
                InterpreterState::Continue(_label) => {
                    // TODO, continue to a label.
                    break;
                }
                InterpreterState::Executing => {
                    // Continue execution
                }
            }
            if i + 1 == self.statements().len() {
                obj = val;
            }
        }

        Ok(obj)
    }
}
