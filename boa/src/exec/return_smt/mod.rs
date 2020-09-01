use super::{Executable, Interpreter, InterpreterState};
use crate::{syntax::ast::node::Return, Result, Value};

impl Executable for Return {
    fn run(&self, interpreter: &mut Interpreter) -> Result<Value> {
        let result = match self.expr() {
            Some(ref v) => v.run(interpreter),
            None => Ok(Value::undefined()),
        };
        // Set flag for return
        interpreter.set_current_state(InterpreterState::Return);
        result
    }
}
