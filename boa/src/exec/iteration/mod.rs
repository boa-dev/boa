//! Iteration node execution.

use super::{Executable, Interpreter};
use crate::{
    builtins::value::{ResultValue, Value},
    environment::lexical_environment::new_declarative_environment,
    syntax::ast::node::ForLoop,
    BoaProfiler,
};

impl Executable for ForLoop {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        // Create the block environment.
        let _timer = BoaProfiler::global().start_event("ForLoop", "exec");
        {
            let env = &mut interpreter.realm_mut().environment;
            env.push(new_declarative_environment(Some(
                env.get_current_environment_ref().clone(),
            )));
        }

        if let Some(init) = self.init() {
            init.run(interpreter)?;
        }

        while self
            .condition()
            .map(|cond| cond.run(interpreter).map(|v| v.is_true()))
            .transpose()?
            .unwrap_or(true)
        {
            self.body().run(interpreter)?;

            if let Some(final_expr) = self.final_expr() {
                final_expr.run(interpreter)?;
            }
        }

        // pop the block env
        let _ = interpreter.realm_mut().environment.pop();

        Ok(Value::undefined())
    }
}
