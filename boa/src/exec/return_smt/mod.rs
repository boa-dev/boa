use super::{Executable, Interpreter};
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
        interpreter.is_return = true;
        result
    }
}
