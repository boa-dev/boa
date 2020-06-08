use super::{Executable, Interpreter, InterpreterState};
use crate::{
    builtins::value::{ResultValue, Value},
    syntax::ast::node::Return,
};

impl Executable for Return {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let result = match self.expr() {
            Some(ref v) => v.run(interpreter),
            None => Ok(Value::undefined()),
        };
        // Set flag for return
        interpreter.set_current_state(InterpreterState::Return(self.label().map(String::from)));
        result
    }
}
