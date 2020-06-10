//! Statement list execution.

use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::StatementList,
    BoaProfiler,
};

impl Executable for StatementList {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let _timer = BoaProfiler::global().start_event("StatementList", "exec");
        let mut obj = Value::null();
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
                _ => {
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
