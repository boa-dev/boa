//! Block statement execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value},
    environment::lexical_environment::new_declarative_environment,
    syntax::ast::node::Block,
    BoaProfiler,
};

impl Executable for Block {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let _timer = BoaProfiler::global().start_event("Block", "exec");
        {
            let env = &mut interpreter.realm_mut().environment;
            env.push(new_declarative_environment(Some(
                env.get_current_environment_ref().clone(),
            )));
        }

        let mut obj = Value::null();
        for statement in self.statements() {
            obj = statement.run(interpreter)?;

            // early return
            if interpreter.is_return {
                break;
            }
        }

        // pop the block env
        let _ = interpreter.realm_mut().environment.pop();

        Ok(obj)
    }
}
