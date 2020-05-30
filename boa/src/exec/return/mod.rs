
use super::{Executable, Interpreter};
use crate::{
    builtins::{
        object::{INSTANCE_PROTOTYPE, PROTOTYPE},
        value::{ResultValue, Value, ValueData},
    },
    syntax::ast::node::{Call, New, Node},
};

impl Executable for Return {
    fn run(&self, interpreter: &mut Interpreter) -> ResultValue {
        let result = match *self.ret() {
            Some(ref v) => v.run(interpreter),
            None => Ok(Value::undefined()),
        };
        // Set flag for return
        interpreter.is_return = true;
        result
    }
}