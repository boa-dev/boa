//! Block statement execution.

use super::{Executable, Interpreter, InterpreterState};
use crate::{
    environment::lexical_environment::new_declarative_environment, syntax::ast::node::Block,
    BoaProfiler, Result, Value,
};

impl Executable for Block {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
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

            match interpreter.get_current_state() {
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
